use crate::error::{Error, Result};
use crate::theme::Theme;
use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgb};
use std::io::Cursor;

const MAX_LINES: usize = 80;
const MAX_COLS: usize = 120;
const GLYPH: u32 = 8;

pub fn render_png(text: &str, theme: &Theme) -> Result<Vec<u8>> {
    let mut lines = prepare_lines(text);
    if lines.is_empty() {
        lines.push("(no output)".into());
    }

    let max_cols = lines.iter().map(|l| l.chars().count()).max().unwrap_or(1);

    let ttf_font = theme
        .font_data
        .as_ref()
        .map(|data| FontRef::try_from_slice(data))
        .transpose()
        .map_err(|e| Error::Image(format!("invalid font: {e}")))?;

    let (cell_w, cell_h) = if let Some(ref font) = ttf_font {
        let scaled = font.as_scaled(PxScale::from(theme.font_size));
        let advance = scaled.h_advance(font.glyph_id('M'));
        let height = scaled.height();
        (advance.ceil() as u32, height.ceil() as u32)
    } else {
        (GLYPH * theme.scale, GLYPH * theme.scale)
    };

    let w = theme.padding * 2 + (max_cols as u32) * cell_w;
    let h = theme.padding * 2 + (lines.len() as u32) * cell_h;

    let mut img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(w, h, theme.bg);

    if let Some(ref font) = ttf_font {
        let scaled = font.as_scaled(PxScale::from(theme.font_size));
        let ascent = scaled.ascent();
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let x = theme.padding as f32 + (col as f32) * cell_w as f32;
                let y = theme.padding as f32 + (row as f32) * cell_h as f32 + ascent;
                stamp_glyph_ttf(&mut img, font, theme.font_size, x, y, ch, theme.fg);
            }
        }
    } else {
        for (row, line) in lines.iter().enumerate() {
            for (col, ch) in line.chars().enumerate() {
                let x = theme.padding + (col as u32) * cell_w;
                let y = theme.padding + (row as u32) * cell_h;
                stamp_glyph(&mut img, x, y, ch, theme.scale, theme.fg);
            }
        }
    }

    let mut buf = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .map_err(|e| Error::Image(format!("encoding screenshot: {e}")))?;
    Ok(buf.into_inner())
}

fn stamp_glyph_ttf(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    font: &FontRef,
    font_size: f32,
    x: f32,
    y: f32,
    ch: char,
    fg: Rgb<u8>,
) {
    let glyph_id = font.glyph_id(ch);
    let glyph = glyph_id.with_scale_and_position(PxScale::from(font_size), ab_glyph::point(x, y));

    if let Some(outlined) = font.outline_glyph(glyph) {
        let bounds = outlined.px_bounds();
        outlined.draw(|px, py, coverage| {
            if coverage < 0.1 {
                return;
            }
            let ix = (bounds.min.x as u32) + px;
            let iy = (bounds.min.y as u32) + py;
            if ix < img.width() && iy < img.height() {
                if coverage >= 0.5 {
                    img.put_pixel(ix, iy, fg);
                } else {
                    // Blend with background
                    let bg = img.get_pixel(ix, iy);
                    let r = (fg.0[0] as f32 * coverage + bg.0[0] as f32 * (1.0 - coverage)) as u8;
                    let g = (fg.0[1] as f32 * coverage + bg.0[1] as f32 * (1.0 - coverage)) as u8;
                    let b = (fg.0[2] as f32 * coverage + bg.0[2] as f32 * (1.0 - coverage)) as u8;
                    img.put_pixel(ix, iy, Rgb([r, g, b]));
                }
            }
        });
    }
}

fn prepare_lines(text: &str) -> Vec<String> {
    let norm = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = norm.lines().map(clamp_line).collect();
    if lines.len() > MAX_LINES {
        lines.truncate(MAX_LINES);
        lines.push("(output truncated)".into());
    }
    lines
}

fn clamp_line(line: &str) -> String {
    let expanded = line.replace('\t', "    ");
    let mut out = String::new();
    for (i, ch) in expanded.chars().enumerate() {
        if i >= MAX_COLS {
            out.push_str("...");
            break;
        }
        if ch.is_ascii() && !ch.is_control() {
            out.push(ch);
        } else {
            out.push('?');
        }
    }
    out
}

fn stamp_glyph(
    img: &mut ImageBuffer<Rgb<u8>, Vec<u8>>,
    ox: u32,
    oy: u32,
    ch: char,
    scale: u32,
    fg: Rgb<u8>,
) {
    let glyph = BASIC_FONTS
        .get(ch)
        .or_else(|| BASIC_FONTS.get('?'))
        .unwrap_or([0; 8]);

    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..8 {
            if bits & (1 << col) == 0 {
                continue;
            }
            for sy in 0..scale {
                for sx in 0..scale {
                    let x = ox + (col as u32) * scale + sx;
                    let y = oy + (row as u32) * scale + sy;
                    if x < img.width() && y < img.height() {
                        img.put_pixel(x, y, fg);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_short_line() {
        assert_eq!(clamp_line("hello"), "hello");
    }

    #[test]
    fn clamp_tabs_expand() {
        assert_eq!(clamp_line("\t"), "    ");
    }

    #[test]
    fn clamp_long_line_truncated() {
        let long = "x".repeat(200);
        let out = clamp_line(&long);
        assert!(out.ends_with("..."));
        assert!(out.len() <= MAX_COLS + 3);
    }

    #[test]
    fn prepare_lines_caps_at_max() {
        let text = "line\n".repeat(MAX_LINES + 50);
        let lines = prepare_lines(&text);
        assert_eq!(lines.len(), MAX_LINES + 1);
        assert_eq!(lines.last().unwrap(), "(output truncated)");
    }

    #[test]
    fn render_png_produces_bytes() {
        let png = render_png("hello world", &Theme::default()).unwrap();
        assert!(png.len() > 100);
        assert_eq!(&png[1..4], b"PNG");
    }
}
