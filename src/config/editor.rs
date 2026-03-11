use super::{AppConfig, config_path, load, save};
use crate::error::{Error, Result, io_err};
use crate::ui;
use owo_colors::OwoColorize;
use std::io::{self, BufRead, Write};
use std::process::Command;

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

pub fn run_config_editor() -> Result<()> {
    let path = config_path()?;
    let mut cfg = load(&path)?;

    if !path.exists() {
        save(&path, &cfg)?;
    }

    let editor = find_editor(&cfg).or_else(pick_editor_menu).ok_or_else(|| {
        Error::Validation("no editor found - set $EDITOR or use `config set --editor`".into())
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
