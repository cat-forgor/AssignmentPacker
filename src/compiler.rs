use crate::error::{Error, Result, io_err};
use std::process::{Command, Output};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{env, io, path::Path, thread};

const RUN_TIMEOUT: Duration = Duration::from_secs(30);

pub struct RunCapture {
    pub command_display: String,
    pub formatted_output: String,
    pub screenshot_text: String,
}

pub fn capture_run(
    c_file: &Path,
    run_command: Option<&str>,
    display_command: &str,
) -> Result<RunCapture> {
    if let Some(cmd) = run_command {
        let output = shell_exec(cmd)?;
        let formatted = format_output(&output);
        let screenshot_text = format!("$ {display_command}\n\n{formatted}");
        return Ok(RunCapture {
            command_display: display_command.to_string(),
            formatted_output: formatted,
            screenshot_text,
        });
    }

    let compiler = detect_compiler().ok_or_else(|| {
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

    let compile = Command::new(compiler)
        .arg(c_file)
        .arg("-o")
        .arg(&bin)
        .output()
        .map_err(|e| io_err(format!("running {compiler}"), e))?;

    if !compile.status.success() {
        return Err(Error::CompileFailed(format_output(&compile)));
    }

    let run_output = run_with_timeout(&bin, RUN_TIMEOUT)?;
    if let Err(e) = std::fs::remove_file(&bin) {
        eprintln!("warning: couldn't clean up temp binary: {e}");
    }

    let formatted = format_output(&run_output);
    let screenshot_text = format!("$ {display_command}\n\n{formatted}");

    Ok(RunCapture {
        command_display: display_command.to_string(),
        formatted_output: formatted,
        screenshot_text,
    })
}

fn shell_exec(command: &str) -> Result<Output> {
    if cfg!(windows) {
        Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(command)
            .output()
            .map_err(|e| io_err(format!("running '{command}'"), e))
    } else {
        Command::new("sh")
            .arg("-c")
            .arg(command)
            .output()
            .map_err(|e| io_err(format!("running '{command}'"), e))
    }
}

fn run_with_timeout(bin: &Path, timeout: Duration) -> Result<Output> {
    let mut child = Command::new(bin)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| io_err(format!("spawning '{}'", bin.display()), e))?;

    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = child
                    .stdout
                    .take()
                    .map(io::read_to_string)
                    .transpose()
                    .map_err(|e| io_err("reading stdout", e))?
                    .unwrap_or_default();
                let stderr = child
                    .stderr
                    .take()
                    .map(io::read_to_string)
                    .transpose()
                    .map_err(|e| io_err("reading stderr", e))?
                    .unwrap_or_default();
                return Ok(Output {
                    status,
                    stdout: stdout.into_bytes(),
                    stderr: stderr.into_bytes(),
                });
            }
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                return Err(Error::Validation(format!(
                    "program timed out after {}s",
                    timeout.as_secs()
                )));
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(io_err("waiting for process", e)),
        }
    }
}

fn detect_compiler() -> Option<&'static str> {
    ["gcc", "clang"]
        .into_iter()
        .find(|c| Command::new(c).arg("--version").output().is_ok())
}

fn format_output(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    let stderr = String::from_utf8_lossy(&output.stderr)
        .trim_end()
        .to_string();

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
