use crate::error::{Result, io_err};
use std::io::{Read, Write};
use std::process::{Command, Output, Stdio};
use std::time::{Duration, Instant};
use std::{io, thread};

pub fn expand_escapes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('0') => out.push('\0'),
                Some('\\') => out.push('\\'),
                Some('x') => {
                    let hi = chars.next();
                    let lo = chars.next();
                    if let (Some(h), Some(l)) = (hi, lo) {
                        let hex: String = [h, l].iter().collect();
                        if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                            out.push(byte as char);
                        } else {
                            out.push_str("\\x");
                            out.push(h);
                            out.push(l);
                        }
                    } else {
                        out.push_str("\\x");
                        if let Some(h) = hi {
                            out.push(h);
                        }
                    }
                }
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expand_basic() {
        assert_eq!(expand_escapes("a\\nb"), "a\nb");
        assert_eq!(expand_escapes("a\\tb"), "a\tb");
        assert_eq!(expand_escapes("a\\\\b"), "a\\b");
    }

    #[test]
    fn expand_cr() {
        assert_eq!(expand_escapes("a\\rb"), "a\rb");
    }

    #[test]
    fn expand_nul() {
        assert_eq!(expand_escapes("a\\0b"), "a\0b");
    }

    #[test]
    fn expand_hex() {
        assert_eq!(expand_escapes("\\x41"), "A");
        assert_eq!(expand_escapes("\\x0a"), "\n");
    }

    #[test]
    fn expand_invalid_hex_passthrough() {
        assert_eq!(expand_escapes("\\xZZ"), "\\xZZ");
        assert_eq!(expand_escapes("\\x4"), "\\x4");
    }
}

pub fn shell_exec_with_input(command: &str, input: Option<&str>, timeout: Duration) -> Result<Output> {
    let (shell, flag): (&str, &[&str]) = if cfg!(windows) {
        ("powershell", &["-NoProfile", "-Command"])
    } else {
        ("sh", &["-c"])
    };

    let stdin_mode = if input.is_some() {
        Stdio::piped()
    } else {
        Stdio::null()
    };

    let mut child = Command::new(shell)
        .args(flag)
        .arg(command)
        .stdin(stdin_mode)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| io_err(format!("running '{command}'"), e))?;

    if let Some(input_str) = input {
        let expanded = expand_escapes(input_str);
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(expanded.as_bytes())
                .map_err(|e| io_err("writing command input to stdin", e))?;
        }
    }

    wait_with_timeout(child, timeout)
}

pub fn run_with_input(bin: &std::path::Path, input: &str, timeout: Duration) -> Result<Output> {
    let mut child = Command::new(bin)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| io_err(format!("spawning '{}'", bin.display()), e))?;

    let expanded = expand_escapes(input);
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(expanded.as_bytes())
            .map_err(|e| io_err("writing program input to stdin", e))?;
    }

    wait_with_timeout(child, timeout)
}

fn wait_with_timeout(mut child: std::process::Child, timeout: Duration) -> Result<Output> {
    let mut stdout_reader = spawn_reader(child.stdout.take());
    let mut stderr_reader = spawn_reader(child.stderr.take());
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = join_reader(stdout_reader.take(), "stdout")?;
                let stderr = join_reader(stderr_reader.take(), "stderr")?;
                return Ok(Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) if start.elapsed() >= timeout => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = join_reader(stdout_reader.take(), "stdout");
                let _ = join_reader(stderr_reader.take(), "stderr");
                return Err(crate::error::Error::Validation(format!(
                    "program timed out after {}s",
                    timeout.as_secs()
                )));
            }
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(e) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = join_reader(stdout_reader.take(), "stdout");
                let _ = join_reader(stderr_reader.take(), "stderr");
                return Err(io_err("waiting for process", e));
            }
        }
    }
}

fn spawn_reader<R>(reader: Option<R>) -> Option<thread::JoinHandle<io::Result<Vec<u8>>>>
where
    R: Read + Send + 'static,
{
    reader.map(|mut r| {
        thread::spawn(move || {
            let mut buf = Vec::new();
            r.read_to_end(&mut buf)?;
            Ok(buf)
        })
    })
}

fn join_reader(
    handle: Option<thread::JoinHandle<io::Result<Vec<u8>>>>,
    stream_name: &str,
) -> Result<Vec<u8>> {
    let Some(handle) = handle else {
        return Ok(Vec::new());
    };
    let result = handle
        .join()
        .map_err(|_| crate::error::Error::Validation(format!("{stream_name} reader thread panicked")))?;
    result.map_err(|e| io_err(format!("reading {stream_name}"), e))
}

pub fn detect_compiler() -> Option<&'static str> {
    ["gcc", "clang"]
        .into_iter()
        .find(|c| Command::new(c).arg("--version").output().is_ok())
}
