use crate::compiler::RunCapture;
use crate::error::{Error, Result};
use image::ImageFormat;

const WATERMARK: &str = "Packed with assignmentpacker, created by Ian Fogarty (catforgor).";

pub struct RtfOptions<'a> {
    pub assignment: &'a str,
    pub name: &'a str,
    pub student_id: &'a str,
    pub c_file_name: &'a str,
    pub code: &'a str,
    pub capture: &'a RunCapture,
    pub screenshot_png: &'a [u8],
    pub watermark: bool,
}

pub fn build_rtf(opts: &RtfOptions<'_>) -> Result<Vec<u8>> {
    let RtfOptions {
        assignment,
        name,
        student_id,
        c_file_name,
        code,
        capture,
        screenshot_png,
        watermark,
    } = opts;
    let img = image::load_from_memory_with_format(screenshot_png, ImageFormat::Png)
        .map_err(|e| Error::Image(format!("reading screenshot: {e}")))?;
    let pw = img.width().max(1) as u64;
    let ph = img.height().max(1) as u64;
    let goal_w = pw.saturating_mul(15);
    let goal_h = ph.saturating_mul(15);
    let hex = hex_wrap(screenshot_png, 64);

    let mut r = String::with_capacity(screenshot_png.len() * 2 + code.len() + 4096);
    r.push_str("{\\rtf1\\ansi\\deff0\n");
    r.push_str("{\\fonttbl{\\f0 Calibri;}{\\f1 Consolas;}}\n");
    r.push_str("\\viewkind4\\uc1\\pard\\sa120\\sl240\\slmult1\\f0\\fs24\n");

    r.push_str("\\b ");
    rtf_escape(&mut r, &format!("{assignment} Submission"), Mode::Inline);
    r.push_str(" \\b0\\par\n");
    rtf_escape(
        &mut r,
        &format!("Student: {name} ({student_id})"),
        Mode::Inline,
    );
    r.push_str("\\par\n");
    rtf_escape(&mut r, &format!("Source file: {c_file_name}"), Mode::Inline);
    r.push_str("\\par\n\\par\n");

    r.push_str("\\b Code\\b0\\par\n");
    r.push_str("{\\pard\\f1\\fs18 ");
    rtf_escape(&mut r, code, Mode::Block);
    r.push_str("\\par}\n\\pard\\f0\\fs24\\par\n");

    r.push_str("\\b Program Run Screenshot\\b0\\par\n");
    rtf_escape(
        &mut r,
        &format!("Command: {}", capture.command_display),
        Mode::Inline,
    );
    r.push_str("\\par\n");
    r.push_str(&format!(
        "{{\\pict\\pngblip\\picw{pw}\\pich{ph}\\picwgoal{goal_w}\\pichgoal{goal_h}\n{hex}}}\n\\par\n"
    ));

    r.push_str("\\b Captured Output (Text)\\b0\\par\n");
    r.push_str("{\\pard\\f1\\fs18 ");
    rtf_escape(&mut r, &capture.formatted_output, Mode::Block);
    r.push_str("\\par}\n");

    if *watermark {
        r.push_str("\\pard\\qc\\f0\\fs16\\i ");
        rtf_escape(&mut r, WATERMARK, Mode::Inline);
        r.push_str(" \\i0\\par\n");
    }
    r.push_str("}\n");

    Ok(r.into_bytes())
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Inline,
    Block,
}

fn rtf_escape(buf: &mut String, text: &str, mode: Mode) {
    for ch in text.chars() {
        match ch {
            '\\' => buf.push_str("\\\\"),
            '{' => buf.push_str("\\{"),
            '}' => buf.push_str("\\}"),
            '\n' if mode == Mode::Block => buf.push_str("\\line\n"),
            '\r' if mode == Mode::Block => {}
            '\t' if mode == Mode::Block => buf.push_str("    "),
            '\n' | '\r' | '\t' => buf.push(' '),
            _ if ch.is_ascii() && !ch.is_control() => buf.push(ch),
            _ => rtf_push_unicode(buf, ch),
        }
    }
}

fn rtf_push_unicode(buf: &mut String, ch: char) {
    use std::fmt::Write;
    let cp = ch as u32;
    if cp <= 0x7FFF {
        let _ = write!(buf, "\\u{cp}?");
    } else if cp <= 0xFFFF {
        let signed = cp as i16;
        let _ = write!(buf, "\\u{signed}?");
    } else {
        let adj = cp - 0x10000;
        let hi = 0xD800 + (adj >> 10);
        let lo = 0xDC00 + (adj & 0x3FF);
        let _ = write!(buf, "\\u{}?\\u{}?", hi as i16, lo as i16);
    }
}

fn hex_wrap(bytes: &[u8], per_line: usize) -> String {
    let per_line = per_line.max(1);
    let mut out = String::with_capacity(bytes.len() * 2 + bytes.len() / per_line + 4);
    for (i, b) in bytes.iter().enumerate() {
        use std::fmt::Write;
        let _ = write!(out, "{b:02x}");
        if (i + 1) % per_line == 0 {
            out.push('\n');
        }
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtf_escapes_special_chars() {
        let mut buf = String::new();
        rtf_escape(&mut buf, r"a\b{c}d", Mode::Inline);
        assert_eq!(buf, r"a\\b\{c\}d");
    }

    #[test]
    fn rtf_block_converts_newlines() {
        let mut buf = String::new();
        rtf_escape(&mut buf, "a\nb", Mode::Block);
        assert_eq!(buf, "a\\line\nb");
    }

    #[test]
    fn rtf_block_expands_tabs() {
        let mut buf = String::new();
        rtf_escape(&mut buf, "\t", Mode::Block);
        assert_eq!(buf, "    ");
    }

    #[test]
    fn rtf_inline_collapses_whitespace() {
        let mut buf = String::new();
        rtf_escape(&mut buf, "a\nb\tc", Mode::Inline);
        assert_eq!(buf, "a b c");
    }

    #[test]
    fn rtf_unicode_escape_basic() {
        let mut buf = String::new();
        rtf_escape(&mut buf, "Jos\u{00e9}", Mode::Inline);
        assert_eq!(buf, "Jos\\u233?");
    }

    #[test]
    fn rtf_unicode_block_escape() {
        let mut buf = String::new();
        rtf_escape(&mut buf, "\u{00e9}", Mode::Block);
        assert_eq!(buf, "\\u233?");
    }

    #[test]
    fn hex_wraps_at_boundary() {
        let out = hex_wrap(&[0xAB, 0xCD, 0xEF, 0x01], 2);
        assert_eq!(out, "abcd\nef01\n");
    }

    #[test]
    fn hex_handles_odd_count() {
        let out = hex_wrap(&[0xFF], 4);
        assert_eq!(out, "ff\n");
    }
}
