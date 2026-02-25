use crate::config;
use crate::error::{Error, Result, io_err};
use image::Rgb;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Clone)]
pub struct Theme {
    pub bg: Rgb<u8>,
    pub fg: Rgb<u8>,
    pub padding: u32,
    pub scale: u32,
    pub font_data: Option<Vec<u8>>,
    pub font_size: f32,
}

#[derive(Deserialize)]
struct ThemeFile {
    bg: Option<String>,
    fg: Option<String>,
    padding: Option<u32>,
    scale: Option<u32>,
    font: Option<String>,
    font_size: Option<f32>,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            bg: Rgb([15, 18, 24]),
            fg: Rgb([128, 255, 170]),
            padding: 16,
            scale: 2,
            font_data: None,
            font_size: 16.0,
        }
    }
}

pub fn resolve(name: Option<&str>) -> Result<Theme> {
    let name = match name {
        Some(n) => n.trim(),
        None => return Ok(Theme::default()),
    };
    if name.is_empty() {
        return Err(Error::Validation("theme name cannot be empty".into()));
    }

    if let Some(theme) = builtin(name) {
        return Ok(theme);
    }

    let themes_dir = config::config_path()?
        .parent()
        .map(|p| p.join("themes"))
        .ok_or_else(|| Error::Validation("can't determine themes directory".into()))?;

    let file = themes_dir.join(format!("{name}.toml"));
    if !file.exists() {
        let available = list_available(&themes_dir);
        return Err(Error::Validation(format!(
            "unknown theme '{name}'\n  built-in: default, light, dracula, monokai, solarized\n  custom:   {available}"
        )));
    }

    load_file(&file)
}

fn builtin(name: &str) -> Option<Theme> {
    Some(match name {
        "default" => Theme::default(),
        "light" => Theme {
            bg: Rgb([255, 255, 255]),
            fg: Rgb([30, 30, 30]),
            ..Theme::default()
        },
        "dracula" => Theme {
            bg: Rgb([40, 42, 54]),
            fg: Rgb([248, 248, 242]),
            ..Theme::default()
        },
        "monokai" => Theme {
            bg: Rgb([39, 40, 34]),
            fg: Rgb([248, 248, 240]),
            ..Theme::default()
        },
        "solarized" => Theme {
            bg: Rgb([0, 43, 54]),
            fg: Rgb([131, 148, 150]),
            ..Theme::default()
        },
        _ => return None,
    })
}

fn load_file(path: &std::path::Path) -> Result<Theme> {
    let content = fs::read_to_string(path).map_err(|e| io_err("reading theme", e))?;
    let raw: ThemeFile =
        toml::from_str(&content).map_err(|e| Error::Validation(format!("bad theme file: {e}")))?;

    let base = Theme::default();
    let scale = raw.scale.unwrap_or(base.scale).clamp(1, 4);
    let padding = raw.padding.unwrap_or(base.padding).min(64);
    let font_size = raw.font_size.unwrap_or(base.font_size).clamp(8.0, 72.0);

    let font_data = if let Some(ref font_path) = raw.font {
        let resolved = if std::path::Path::new(font_path).is_absolute() {
            std::path::PathBuf::from(font_path)
        } else {
            path.parent()
                .ok_or_else(|| Error::Validation("can't resolve font path".into()))?
                .join(font_path)
        };
        let data = fs::read(&resolved)
            .map_err(|e| io_err(format!("reading font '{}'", resolved.display()), e))?;
        Some(data)
    } else {
        None
    };

    Ok(Theme {
        bg: raw
            .bg
            .as_deref()
            .map(parse_hex)
            .transpose()?
            .unwrap_or(base.bg),
        fg: raw
            .fg
            .as_deref()
            .map(parse_hex)
            .transpose()?
            .unwrap_or(base.fg),
        padding,
        scale,
        font_data,
        font_size,
    })
}

fn parse_hex(s: &str) -> Result<Rgb<u8>> {
    let s = s.strip_prefix('#').unwrap_or(s);
    if s.len() != 6 {
        return Err(Error::Validation(format!(
            "invalid hex color '{s}', expected 6 hex digits (e.g. #1a2b3c)"
        )));
    }
    let r = u8::from_str_radix(&s[0..2], 16);
    let g = u8::from_str_radix(&s[2..4], 16);
    let b = u8::from_str_radix(&s[4..6], 16);
    match (r, g, b) {
        (Ok(r), Ok(g), Ok(b)) => Ok(Rgb([r, g, b])),
        _ => Err(Error::Validation(format!(
            "invalid hex color '{s}', expected 6 hex digits (e.g. #1a2b3c)"
        ))),
    }
}

fn list_available(themes_dir: &std::path::Path) -> String {
    let Ok(walker) = fs::read_dir(themes_dir) else {
        return "(none, create themes in ~/.config/assignment_packer/themes/)".into();
    };

    let mut names = Vec::new();
    collect_themes(themes_dir, themes_dir, &mut names);
    // Also grab top-level entries if read_dir succeeded but walkdir missed them
    drop(walker);

    names.sort();
    names.dedup();
    if names.is_empty() {
        "(none, create themes in ~/.config/assignment_packer/themes/)".into()
    } else {
        names.join(", ")
    }
}

fn collect_themes(base: &std::path::Path, dir: &std::path::Path, names: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_themes(base, &path, names);
        } else if let Some(ext) = path.extension()
            && ext == "toml"
            && let Ok(rel) = path.strip_prefix(base)
        {
            let name = rel.with_extension("");
            if let Some(s) = name.to_str() {
                names.push(s.replace('\\', "/"));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hex_valid() {
        let c = parse_hex("#1a2b3c").unwrap();
        assert_eq!(c, Rgb([0x1a, 0x2b, 0x3c]));
    }

    #[test]
    fn parse_hex_no_hash() {
        let c = parse_hex("ff00aa").unwrap();
        assert_eq!(c, Rgb([0xff, 0x00, 0xaa]));
    }

    #[test]
    fn parse_hex_rejects_short() {
        assert!(parse_hex("#fff").is_err());
    }

    #[test]
    fn parse_hex_rejects_garbage() {
        assert!(parse_hex("nothex").is_err());
    }

    #[test]
    fn builtin_default_exists() {
        assert!(builtin("default").is_some());
    }

    #[test]
    fn builtin_unknown_returns_none() {
        assert!(builtin("nonexistent").is_none());
    }

    #[test]
    fn resolve_none_gives_default() {
        let t = resolve(None).unwrap();
        let d = Theme::default();
        assert_eq!(t.bg, d.bg);
        assert_eq!(t.fg, d.fg);
    }

    #[test]
    fn resolve_builtin_works() {
        let t = resolve(Some("dracula")).unwrap();
        assert_eq!(t.bg, Rgb([40, 42, 54]));
    }
}
