use crate::cli::{ConfigCommand, ConfigSetArgs};
use crate::error::{Error, Result, io_err};
use crate::ui;
use crate::validate::clean_name;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const DIR_NAME: &str = "assignment_packer";
const FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub name: Option<String>,
    #[serde(rename = "id", alias = "student_id")]
    pub student_id: Option<String>,
    pub output_dir: Option<PathBuf>,
    #[serde(alias = "autoDoc")]
    pub auto_doc: Option<bool>,
    #[serde(alias = "runCommand")]
    pub run_command: Option<String>,
    #[serde(alias = "runDisplayTemplate")]
    pub run_display_template: Option<String>,
    pub theme: Option<String>,
    pub editor: Option<String>,
    pub watermark: Option<bool>,
}

pub fn run_init() -> Result<()> {
    let path = config_path()?;
    let mut cfg = load(&path)?;

    ui::header("ap setup");
    eprintln!();

    let name = prompt("Student name (e.g. JoeBloggs)")?;
    if !name.is_empty() {
        cfg.name = Some(clean_name(&name, "name")?);
    }

    let id = prompt("Student ID")?;
    if !id.is_empty() {
        cfg.student_id = Some(clean_name(&id, "student ID")?);
    }

    let theme = prompt("Theme (default, light, dracula, monokai, solarized, custom)")?;
    if !theme.is_empty() {
        cfg.theme = Some(clean_name(&theme, "theme")?);
    }

    let auto = prompt("Enable auto-doc? [Y/n] ")?;
    let auto = auto.trim().to_ascii_lowercase();
    if auto.is_empty() || auto == "y" || auto == "yes" {
        cfg.auto_doc = Some(true);
    } else {
        cfg.auto_doc = Some(false);
    }

    save(&path, &cfg)?;
    eprintln!();
    ui::done("config saved");
    print_config(&path, &cfg);
    Ok(())
}

fn prompt(label: &str) -> Result<String> {
    eprint!("  {} ", label.bold());
    io::stderr()
        .flush()
        .map_err(|e| io_err("flushing stderr", e))?;
    let mut line = String::new();
    io::stdin()
        .lock()
        .read_line(&mut line)
        .map_err(|e| io_err("reading input", e))?;
    Ok(line.trim().to_string())
}

pub fn run_config_command(command: Option<ConfigCommand>) -> Result<()> {
    match command {
        None | Some(ConfigCommand::Show) => {
            let path = config_path()?;
            let cfg = load(&path)?;
            print_config(&path, &cfg);
            Ok(())
        }
        Some(ConfigCommand::Path) => {
            let path = config_path()?;
            println!("{}", path.display());
            Ok(())
        }
        Some(ConfigCommand::Set(args)) => apply_set(args),
        Some(ConfigCommand::Reset) => {
            let path = config_path()?;
            if path.exists() {
                fs::remove_file(&path).map_err(|e| io_err("can't remove config", e))?;
            }
            ui::done(&format!("config reset: {}", path.display()));
            Ok(())
        }
        Some(ConfigCommand::Editor) => run_config_editor(),
    }
}

fn apply_set(args: ConfigSetArgs) -> Result<()> {
    let path = config_path()?;
    let mut cfg = load(&path)?;
    let mut changed = false;

    if let Some(name) = args.name {
        cfg.name = Some(clean_name(&name, "name")?);
        changed = true;
    }
    if let Some(id) = args.student_id {
        cfg.student_id = Some(clean_name(&id, "student ID")?);
        changed = true;
    }
    if let Some(dir) = args.output_dir {
        if !dir.is_dir() {
            return Err(Error::Validation(format!(
                "not a directory: '{}'",
                dir.display()
            )));
        }
        cfg.output_dir = Some(dir);
        changed = true;
    }
    if let Some(v) = args.auto_doc {
        cfg.auto_doc = Some(v);
        changed = true;
    }
    if args.clear_run_command {
        cfg.run_command = None;
        changed = true;
    }
    if let Some(cmd) = args.run_command {
        let trimmed = cmd.trim();
        if trimmed.is_empty() {
            return Err(Error::Validation("run-command cannot be blank".into()));
        }
        cfg.run_command = Some(trimmed.to_string());
        changed = true;
    }
    if args.clear_run_display_template {
        cfg.run_display_template = None;
        changed = true;
    }
    if let Some(tpl) = args.run_display_template {
        let trimmed = tpl.trim();
        if trimmed.is_empty() {
            return Err(Error::Validation(
                "run-display-template cannot be blank".into(),
            ));
        }
        cfg.run_display_template = Some(trimmed.to_string());
        changed = true;
    }
    if args.clear_theme {
        cfg.theme = None;
        changed = true;
    }
    if let Some(t) = args.theme {
        let trimmed = t.trim();
        if trimmed.is_empty() {
            return Err(Error::Validation("theme cannot be blank".into()));
        }
        cfg.theme = Some(trimmed.to_string());
        changed = true;
    }
    if args.clear_editor {
        cfg.editor = None;
        changed = true;
    }
    if let Some(e) = args.editor {
        let trimmed = e.trim();
        if trimmed.is_empty() {
            return Err(Error::Validation("editor cannot be blank".into()));
        }
        cfg.editor = Some(trimmed.to_string());
        changed = true;
    }
    if let Some(v) = args.watermark {
        cfg.watermark = Some(v);
        changed = true;
    }
    if !changed {
        return Err(Error::Validation(
            "nothing to update — pass at least one flag (see `config set --help`)".into(),
        ));
    }

    save(&path, &cfg)?;
    ui::done("config updated");
    print_config(&path, &cfg);
    Ok(())
}

fn print_config(path: &Path, cfg: &AppConfig) {
    let val = |v: Option<&str>| v.unwrap_or("-").to_string();

    ui::kv("path", &path.to_string_lossy());
    ui::kv("name", &val(cfg.name.as_deref()));
    ui::kv("id", &val(cfg.student_id.as_deref()));
    ui::kv(
        "output_dir",
        &cfg.output_dir
            .as_ref()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_else(|| "-".into()),
    );
    ui::kv(
        "auto_doc",
        match cfg.auto_doc {
            Some(true) => "true",
            Some(false) => "false",
            None => "-",
        },
    );
    ui::kv("run_command", &val(cfg.run_command.as_deref()));
    ui::kv(
        "run_display_template",
        &val(cfg.run_display_template.as_deref()),
    );
    ui::kv("theme", &val(cfg.theme.as_deref()));
    ui::kv("editor", &val(cfg.editor.as_deref()));
    ui::kv(
        "watermark",
        match cfg.watermark {
            Some(true) => "true",
            Some(false) => "false",
            None => "-",
        },
    );
}

const KNOWN_EDITORS: &[&str] = &[
    "code --wait",
    "codium --wait",
    "zed --wait",
    "subl --wait",
    "hx",
    "nvim",
    "vim",
    "nano",
    "micro",
    "emacs -nw",
    "kak",
    "notepad",
];

fn run_config_editor() -> Result<()> {
    let path = config_path()?;
    let mut cfg = load(&path)?;

    // Ensure config file exists on disk
    if !path.exists() {
        save(&path, &cfg)?;
    }

    let editor = find_editor(&cfg).or_else(pick_editor_menu).ok_or_else(|| {
        Error::Validation("no editor found — set $EDITOR or use `config set --editor`".into())
    })?;

    ui::step(&format!("Opening {} ...", path.display()));
    let parts: Vec<&str> = editor.split_whitespace().collect();
    let status = Command::new(parts[0])
        .args(&parts[1..])
        .arg(&path)
        .status()
        .map_err(|e| io_err(format!("launching '{}'", parts[0]), e))?;

    if !status.success() {
        ui::warn(&format!("editor exited with {}", status));
    }

    // Remember the editor for next time
    if cfg.editor.as_deref() != Some(&editor) {
        cfg.editor = Some(editor);
        save(&path, &cfg)?;
    }

    ui::done("config editor closed");
    Ok(())
}

fn find_editor(cfg: &AppConfig) -> Option<String> {
    let candidates = cfg
        .editor
        .iter()
        .cloned()
        .chain(std::env::var("VISUAL").ok())
        .chain(std::env::var("EDITOR").ok());

    for candidate in candidates {
        let trimmed = candidate.trim().to_string();
        if !trimmed.is_empty() && editor_exists(&trimmed) {
            return Some(trimmed);
        }
    }

    for &candidate in KNOWN_EDITORS {
        if editor_exists(candidate) {
            return Some(candidate.to_string());
        }
    }

    None
}

fn editor_exists(cmd: &str) -> bool {
    let program = match cmd.split_whitespace().next() {
        Some(p) => p,
        None => return false,
    };
    Command::new(program)
        .arg("--version")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}

fn pick_editor_menu() -> Option<String> {
    eprintln!("  No editor detected. Pick one:\n");
    for (i, &name) in KNOWN_EDITORS.iter().enumerate() {
        eprintln!("    {}  {}", format!("[{}]", i + 1).bold(), name);
    }
    eprintln!();

    loop {
        eprint!("  Choice: ");
        let _ = io::stderr().flush();
        let mut line = String::new();
        if io::stdin().lock().read_line(&mut line).is_err() {
            return None;
        }
        if let Ok(n) = line.trim().parse::<usize>()
            && n >= 1
            && n <= KNOWN_EDITORS.len()
        {
            return Some(KNOWN_EDITORS[n - 1].to_string());
        }
        eprintln!("  invalid choice, enter 1-{}", KNOWN_EDITORS.len());
    }
}

pub fn config_path() -> Result<PathBuf> {
    let base = dirs::config_dir()
        .ok_or_else(|| Error::Validation("can't determine user config directory".into()))?;
    Ok(base.join(DIR_NAME).join(FILE_NAME))
}

pub fn load(path: &Path) -> Result<AppConfig> {
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(path).map_err(|e| io_err("reading config", e))?;
    toml::from_str(&content).map_err(|e| Error::Validation(format!("bad config: {e}")))
}

pub fn save(path: &Path, cfg: &AppConfig) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| io_err("creating config directory", e))?;
    }
    let content = toml::to_string_pretty(cfg)
        .map_err(|e| Error::Validation(format!("serializing config: {e}")))?;
    fs::write(path, content).map_err(|e| io_err("writing config", e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_round_trips() {
        let cfg = AppConfig::default();
        let s = toml::to_string_pretty(&cfg).unwrap();
        let parsed: AppConfig = toml::from_str(&s).unwrap();
        assert!(parsed.name.is_none());
    }

    #[test]
    fn config_with_values_round_trips() {
        let cfg = AppConfig {
            name: Some("Alice".into()),
            student_id: Some("12345".into()),
            auto_doc: Some(true),
            ..Default::default()
        };
        let s = toml::to_string_pretty(&cfg).unwrap();
        let parsed: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(parsed.name.as_deref(), Some("Alice"));
        assert_eq!(parsed.auto_doc, Some(true));
    }
}
