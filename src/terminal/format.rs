use std::process::Output;

pub fn format_output(output: &Output) -> String {
    format_output_with_cols(output, None)
}

pub fn format_output_with_cols(output: &Output, cols: Option<usize>) -> String {
    let process = |text: &[u8]| -> String {
        let s = String::from_utf8_lossy(text);
        match cols {
            Some(c) => super::emulate::process_with_cols(&s, c),
            None => super::emulate::process(&s),
        }
    };
    let stdout = process(&output.stdout).trim_end().to_string();
    let stderr = process(&output.stderr).trim_end().to_string();

    let success = output.status.code() == Some(0);

    if success && stderr.is_empty() {
        if stdout.is_empty() {
            return "(no output)".into();
        }
        return stdout;
    }

    let mut parts = Vec::new();
    if !stdout.is_empty() {
        parts.push(format!("STDOUT\n{stdout}"));
    }
    if !stderr.is_empty() {
        parts.push(format!("STDERR\n{stderr}"));
    }
    if parts.is_empty() {
        parts.push("(no output)".into());
    }

    let exit = output
        .status
        .code()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "killed".into());
    parts.push(format!("Exit code: {exit}"));
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::ExitStatus;

    fn make_output(stdout: &str, stderr: &str, code: i32) -> Output {
        #[cfg(unix)]
        let status = {
            use std::os::unix::process::ExitStatusExt;
            ExitStatus::from_raw(code << 8)
        };
        #[cfg(windows)]
        let status = {
            use std::os::windows::process::ExitStatusExt;
            ExitStatus::from_raw(code as u32)
        };
        Output {
            status,
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn clean_success_no_headers() {
        let out = format_output(&make_output("hello world", "", 0));
        assert_eq!(out, "hello world");
    }

    #[test]
    fn error_shows_headers() {
        let out = format_output(&make_output("", "oops", 1));
        assert!(out.contains("STDERR"));
        assert!(out.contains("Exit code: 1"));
    }

    #[test]
    fn success_with_stderr_shows_headers() {
        let out = format_output(&make_output("ok", "warning", 0));
        assert!(out.contains("STDOUT"));
        assert!(out.contains("STDERR"));
    }

    #[test]
    fn no_output_shows_placeholder() {
        let out = format_output(&make_output("", "", 0));
        assert_eq!(out, "(no output)");
    }
}
