pub mod capture;
pub mod emulate;
pub mod exec;
pub mod format;

use crate::error::{Error, Result, io_err};
use crate::ui;
use std::env;
use std::process::Command;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const PTY_COLS: usize = 120;

pub struct RunCapture {
    pub command_display: String,
    pub formatted_output: String,
    pub screenshot_text: String,
}

pub fn capture_run(
    c_file: &std::path::Path,
    run_command: Option<&str>,
    display_command: &str,
    input: Option<&str>,
    timeout: Option<u64>,
) -> Result<RunCapture> {
    let timeout = timeout
        .map(|s| Duration::from_secs(s.clamp(5, 300)))
        .unwrap_or(DEFAULT_TIMEOUT);

    if let Some(cmd) = run_command {
        let output = exec::shell_exec_with_input(cmd, input, timeout)?;
        let formatted = format::format_output(&output);
        let screenshot_text = format!("$ {display_command}\n\n{formatted}");
        return Ok(RunCapture {
            command_display: display_command.to_string(),
            formatted_output: formatted,
            screenshot_text,
        });
    }

    let compiler = exec::detect_compiler().ok_or_else(|| {
        Error::Validation("no C compiler found (gcc/clang), use --run-command".into())
    })?;

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let bin_name = if cfg!(windows) {
        format!("ap_run_{ts}_{}.exe", std::process::id())
    } else {
        format!("ap_run_{ts}_{}", std::process::id())
    };
    let bin = env::temp_dir().join(bin_name);

    ui::step(&format!("Compiling with {compiler}..."));

    let compile = Command::new(compiler)
        .arg(c_file)
        .arg("-o")
        .arg(&bin)
        .output()
        .map_err(|e| io_err(format!("running {compiler}"), e))?;

    if !compile.status.success() {
        return Err(Error::CompileFailed(format::format_output(&compile)));
    }

    let (run_output, pty_cols) = if let Some(input_str) = input {
        (exec::run_with_input(&bin, input_str, timeout)?, None)
    } else {
        (capture::run_interactive(&bin, timeout)?, Some(PTY_COLS))
    };

    if let Err(e) = std::fs::remove_file(&bin) {
        eprintln!("warning: couldn't clean up temp binary: {e}");
    }

    let formatted = format::format_output_with_cols(&run_output, pty_cols);
    let screenshot_text = format!("$ {display_command}\n\n{formatted}");

    Ok(RunCapture {
        command_display: display_command.to_string(),
        formatted_output: formatted,
        screenshot_text,
    })
}
