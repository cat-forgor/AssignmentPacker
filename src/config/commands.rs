use super::{AppConfig, config_path, load, save};
use super::editor::run_config_editor;
use crate::cli::{ConfigCommand, ConfigSetArgs};
use crate::error::{Error, Result, io_err};
use crate::ui;
use crate::validate::clean_name;
use owo_colors::OwoColorize;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::Path;

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
    if args.clear_input {
        cfg.input = None;
        changed = true;
    }
    if let Some(inp) = args.input {
        cfg.input = Some(inp);
        changed = true;
    }
    if let Some(t) = args.timeout {
        if !(5..=300).contains(&t) {
            return Err(Error::Validation("timeout must be 5-300 seconds".into()));
        }
        cfg.timeout = Some(t);
        changed = true;
    }
    if !changed {
        return Err(Error::Validation(
            "nothing to update - pass at least one flag (see `config set --help`)".into(),
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
    ui::kv("input", &val(cfg.input.as_deref()));
    ui::kv(
        "timeout",
        &cfg.timeout
            .map(|t| format!("{t}s"))
            .unwrap_or_else(|| "-".into()),
    );
}
