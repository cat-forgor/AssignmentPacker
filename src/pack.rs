use crate::cli::Cli;
use crate::compiler;
use crate::config;
use crate::error::{Error, Result, io_err};
use crate::fs as afs;
use crate::rtf;
use crate::screenshot;
use crate::theme;
use crate::ui;
use crate::validate::{clean_name, parse_assignment, render_display_command};
use owo_colors::OwoColorize;
use std::path::PathBuf;
use std::{env, fs};

pub fn run_pack(cli: Cli) -> Result<()> {
    let cfg_path = config::config_path()?;
    let cfg = config::load(&cfg_path)?;

    if cli.auto_doc && cli.doc_file.is_some() {
        return Err(Error::Validation(
            "--doc-file and --auto-doc are mutually exclusive".into(),
        ));
    }

    let (assignment, num) = parse_assignment(
        cli.assignment
            .as_deref()
            .ok_or_else(|| Error::Validation("missing --assignment (-a)".into()))?,
    )?;
    let name = clean_name(
        &cli.name
            .or_else(|| cfg.name.clone())
            .ok_or_else(|| Error::Validation("missing --name (or set in config)".into()))?,
        "name",
    )?;
    let student_id = clean_name(
        &cli.student_id
            .or_else(|| cfg.student_id.clone())
            .ok_or_else(|| Error::Validation("missing --id (or set in config)".into()))?,
        "student ID",
    )?;

    let c_file = afs::resolve_c_file(cli.c_file.as_deref())?;
    afs::check_extension(&c_file, &["c"], "C source")?;

    let auto_doc = cli.auto_doc || (cli.doc_file.is_none() && cfg.auto_doc.unwrap_or(false));

    if !auto_doc && cli.run_command.is_some() {
        return Err(Error::Validation(
            "--run-command requires --auto-doc".into(),
        ));
    }
    if !auto_doc && cli.run_display_template.is_some() {
        return Err(Error::Validation(
            "--run-display-template requires --auto-doc".into(),
        ));
    }
    if !auto_doc && cli.theme.is_some() {
        return Err(Error::Validation("--theme requires --auto-doc".into()));
    }

    let out_dir = cli
        .output_dir
        .or_else(|| cfg.output_dir.clone())
        .unwrap_or_else(|| PathBuf::from("."));
    if !out_dir.is_dir() {
        return Err(Error::Validation(format!(
            "output directory not found: '{}'",
            out_dir.display()
        )));
    }

    ui::header(&format!(
        "Packing {} for {} ({})",
        assignment.bold(),
        name,
        student_id,
    ));

    let expected_doc = format!("{assignment}_{name}_{student_id}.doc");
    let manual_doc = if auto_doc {
        None
    } else {
        let path = afs::resolve_doc_file(cli.doc_file.as_deref(), &expected_doc)?;
        afs::check_extension(&path, &["doc"], "Word document")?;
        Some(path)
    };

    let run_command = if auto_doc {
        cli.run_command.or_else(|| cfg.run_command.clone())
    } else {
        None
    };
    let run_tpl = if auto_doc {
        cli.run_display_template
            .or_else(|| cfg.run_display_template.clone())
    } else {
        None
    };

    let folder = format!("{assignment}_{name}_{student_id}_Submission");
    let sub_dir = out_dir.join(&folder);
    let zip_path = out_dir.join(format!("{folder}.zip"));

    afs::prepare_output(&sub_dir, &zip_path, cli.force)?;
    fs::create_dir_all(&sub_dir)
        .map_err(|e| io_err(format!("creating {}", sub_dir.display()), e))?;

    ui::step("Copying files...");
    let c_name = afs::file_name(&c_file)?;
    let cwd = env::current_dir().map_err(|e| io_err("current directory", e))?;
    afs::copy_non_binary_files(&cwd, &sub_dir)?;

    let c_dest = sub_dir.join(c_name);
    let c_in_cwd = c_file
        .parent()
        .is_some_and(|p| fs::canonicalize(p).ok() == fs::canonicalize(&cwd).ok());
    if !c_in_cwd {
        fs::copy(&c_file, &c_dest)
            .map_err(|e| io_err(format!("copying {}", c_file.display()), e))?;
    }

    let doc_dest = sub_dir.join(&expected_doc);
    if auto_doc {
        let display_cmd = render_display_command(
            run_tpl.as_deref(),
            &assignment,
            num,
            &name,
            &student_id,
            &c_file,
        )?;

        ui::step("Compiling...");
        let capture = compiler::capture_run(&c_file, run_command.as_deref(), &display_cmd)?;

        ui::step("Rendering screenshot...");
        let code = afs::read_text_lossy(&c_file)?;
        let theme_name = cli.theme.as_deref().or(cfg.theme.as_deref());
        let theme = theme::resolve(theme_name)?;
        let png = screenshot::render_png(&capture.screenshot_text, &theme)?;

        ui::step("Generating doc...");
        let doc = rtf::build_rtf(&rtf::RtfOptions {
            assignment: &assignment,
            name: &name,
            student_id: &student_id,
            c_file_name: c_name,
            code: &code,
            capture: &capture,
            screenshot_png: &png,
            watermark: !cli.no_watermark && cfg.watermark.unwrap_or(true),
        })?;
        fs::write(&doc_dest, doc)
            .map_err(|e| io_err(format!("writing {}", doc_dest.display()), e))?;
    } else if let Some(src) = manual_doc {
        if afs::paths_equal(&src, &doc_dest) {
            return Err(Error::Validation(
                "doc source and destination resolve to the same file".into(),
            ));
        }
        fs::copy(&src, &doc_dest).map_err(|e| io_err(format!("copying {}", src.display()), e))?;
    } else {
        ui::warn("no .doc included, pass --auto-doc or --doc-file");
    }

    ui::step("Zipping...");
    afs::create_zip(&sub_dir, &zip_path)?;

    eprintln!();
    ui::success(&format!("Created {}", sub_dir.display()));
    ui::success(&format!("Zipped  {}", zip_path.display()));
    if auto_doc {
        ui::success(&format!("Doc     {}", doc_dest.display()));
    }

    Ok(())
}
