use crate::error::{Error, Result, io_err};
use portable_pty::{CommandBuilder, PtySize, native_pty_system};
use std::io::{Read, Write};
use std::process::Output;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, mpsc};
use std::time::{Duration, Instant};
use std::{env, io, thread};

struct RawModeGuard {
    #[cfg(windows)]
    original_mode: u32,
    #[cfg(unix)]
    original_termios: libc::termios,
}

impl RawModeGuard {
    fn enable() -> Option<Self> {
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            let handle = io::stdin().as_raw_handle();
            let mut mode: u32 = 0;
            unsafe {
                if windows_sys::Win32::System::Console::GetConsoleMode(handle as _, &mut mode) == 0
                {
                    return None;
                }
                let raw = mode
                    & !(windows_sys::Win32::System::Console::ENABLE_ECHO_INPUT
                        | windows_sys::Win32::System::Console::ENABLE_LINE_INPUT);
                if windows_sys::Win32::System::Console::SetConsoleMode(handle as _, raw) == 0 {
                    return None;
                }
            }
            Some(Self {
                original_mode: mode,
            })
        }
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = io::stdin().as_raw_fd();
            let mut termios: libc::termios = unsafe { std::mem::zeroed() };
            if unsafe { libc::tcgetattr(fd, &mut termios) } != 0 {
                return None;
            }
            let original = termios;
            termios.c_lflag &= !(libc::ECHO | libc::ICANON);
            if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &termios) } != 0 {
                return None;
            }
            Some(Self {
                original_termios: original,
            })
        }
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            let handle = io::stdin().as_raw_handle();
            unsafe {
                windows_sys::Win32::System::Console::SetConsoleMode(
                    handle as _,
                    self.original_mode,
                );
            }
        }
        #[cfg(unix)]
        {
            use std::os::unix::io::AsRawFd;
            let fd = io::stdin().as_raw_fd();
            unsafe {
                libc::tcsetattr(fd, libc::TCSANOW, &self.original_termios);
            }
        }
    }
}

const MAX_TRANSCRIPT: usize = 4 * 1024 * 1024;

pub fn run_interactive(bin: &std::path::Path, timeout: Duration) -> Result<Output> {
    let eof_key = if cfg!(windows) { "Ctrl+Z" } else { "Ctrl+D" };
    eprintln!("  Program is running. If it doesn't exit on its own, press {eof_key}.\n");

    let _raw_guard = RawModeGuard::enable();

    let pty_system = native_pty_system();
    let pty_size = PtySize {
        rows: 40,
        cols: 120,
        pixel_width: 0,
        pixel_height: 0,
    };
    let pair = pty_system
        .openpty(pty_size)
        .map_err(|e| Error::Validation(format!("opening PTY: {e}")))?;

    let mut cmd = CommandBuilder::new(bin.to_string_lossy().as_ref());
    cmd.cwd(env::current_dir().map_err(|e| io_err("current directory", e))?);
    let mut child = pair
        .slave
        .spawn_command(cmd)
        .map_err(|e| Error::Validation(format!("spawning '{}': {e}", bin.display())))?;
    drop(pair.slave);

    let absolute_deadline = Instant::now() + timeout.saturating_mul(3);

    let mut master_reader = pair
        .master
        .try_clone_reader()
        .map_err(|e| Error::Validation(format!("opening PTY reader: {e}")))?;
    let mut master_writer = pair
        .master
        .take_writer()
        .map_err(|e| Error::Validation(format!("opening PTY writer: {e}")))?;

    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let output_handle = thread::spawn(move || -> io::Result<()> {
        let mut buf = [0_u8; 4096];
        loop {
            match master_reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    let chunk = strip_cpr(&buf[..n]);
                    if !chunk.is_empty() {
                        io::stdout().write_all(&chunk)?;
                        io::stdout().flush()?;
                    }
                    if tx.send(chunk.clone()).is_err() {
                        break;
                    }
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::Interrupted {
                        continue;
                    }
                    break;
                }
            }
        }
        Ok(())
    });

    let stdin_open = Arc::new(AtomicBool::new(true));
    let stdin_open_for_thread = Arc::clone(&stdin_open);
    let _stdin_handle = thread::spawn(move || {
        let stdin = io::stdin();
        let mut locked = stdin.lock();
        let mut buf = [0_u8; 256];
        loop {
            match locked.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if master_writer.write_all(&buf[..n]).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
        stdin_open_for_thread.store(false, Ordering::Relaxed);
    });

    let mut transcript = Vec::new();
    let mut timeout_start: Option<Instant> = None;
    let status = loop {
        drain_chunks(&rx, &mut transcript);
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => {
                if Instant::now() >= absolute_deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    drain_chunks_for(&rx, &mut transcript, Duration::from_millis(120));
                    return Err(Error::Validation(format!(
                        "program timed out after {}s (absolute limit)",
                        timeout.as_secs() * 3
                    )));
                }
                if stdin_open.load(Ordering::Relaxed) {
                    timeout_start = None;
                } else {
                    let started = timeout_start.get_or_insert_with(Instant::now);
                    if started.elapsed() >= timeout {
                        let _ = child.kill();
                        let _ = child.wait();
                        drain_chunks_for(&rx, &mut transcript, Duration::from_millis(120));
                        return Err(Error::Validation(format!(
                            "program timed out after {}s",
                            timeout.as_secs()
                        )));
                    }
                }
                thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                drain_chunks_for(&rx, &mut transcript, Duration::from_millis(120));
                return Err(Error::Validation(format!("waiting for PTY process: {e}")));
            }
        }
    };

    drain_chunks_for(&rx, &mut transcript, Duration::from_millis(180));

    // Give the reader thread a moment to finish, then join it
    let join_deadline = Instant::now() + Duration::from_secs(2);
    while !output_handle.is_finished() && Instant::now() < join_deadline {
        drain_chunks(&rx, &mut transcript);
        thread::sleep(Duration::from_millis(20));
    }
    if output_handle.is_finished() {
        let reader_result = output_handle
            .join()
            .map_err(|_| Error::Validation("PTY output reader thread panicked".into()))?;
        reader_result.map_err(|e| io_err("reading PTY output", e))?;
    }

    if transcript.len() >= MAX_TRANSCRIPT {
        crate::ui::warn("program output exceeded 4 MB, transcript was truncated");
    }

    Ok(Output {
        status: portable_status_to_std(status.exit_code()),
        stdout: transcript,
        stderr: Vec::new(),
    })
}

#[cfg(windows)]
fn portable_status_to_std(code: u32) -> std::process::ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(code)
}

#[cfg(unix)]
fn portable_status_to_std(code: u32) -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw((code as i32) << 8)
}

fn drain_chunks(rx: &mpsc::Receiver<Vec<u8>>, transcript: &mut Vec<u8>) {
    while let Ok(chunk) = rx.try_recv() {
        let remaining = MAX_TRANSCRIPT.saturating_sub(transcript.len());
        if remaining > 0 {
            let take = chunk.len().min(remaining);
            transcript.extend_from_slice(&chunk[..take]);
        }
    }
}

fn drain_chunks_for(rx: &mpsc::Receiver<Vec<u8>>, transcript: &mut Vec<u8>, budget: Duration) {
    let start = Instant::now();
    loop {
        drain_chunks(rx, transcript);
        if start.elapsed() >= budget {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn strip_cpr(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len());
    let mut i = 0;
    while i < data.len() {
        if data[i] == 0x1b && i + 1 < data.len() && data[i + 1] == b'[' {
            let start = i;
            i += 2;
            while i < data.len() && (data[i].is_ascii_digit() || data[i] == b';') {
                i += 1;
            }
            if i < data.len() && data[i] == b'R' {
                i += 1; // skip CPR
            } else {
                out.extend_from_slice(&data[start..i]);
            }
        } else {
            out.push(data[i]);
            i += 1;
        }
    }
    out
}
