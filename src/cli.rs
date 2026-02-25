use clap::{ArgAction, Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    version,
    about = "Packs C assignment submissions for Canvas upload.",
    after_help = "\x1b[1mExamples:\x1b[0m
  ap -a 7 -n JoeBloggs -i 123456789 -c main.c --auto-doc
  ap -a 7 -n JoeBloggs -i 123456789 -c main.c -d Assignment7_JoeBloggs_123456789.doc
  ap -a 7 -c main.c --auto-doc       # uses saved name/id from config
  ap init                            # interactive first-time setup
  ap config show                     # view saved defaults"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    #[arg(
        long,
        short = 'a',
        help = "Assignment number or label (e.g. 7 or Assignment7)"
    )]
    pub assignment: Option<String>,

    #[arg(long, short = 'n', help = "Student name (e.g. JoeBloggs)")]
    pub name: Option<String>,

    #[arg(long = "id", short = 'i', help = "Student ID")]
    pub student_id: Option<String>,

    #[arg(long = "c-file", short = 'c', help = "Path to the .c source file")]
    pub c_file: Option<PathBuf>,

    #[arg(long = "doc-file", short = 'd', help = "Path to the .doc file")]
    pub doc_file: Option<PathBuf>,

    #[arg(
        long = "auto-doc",
        action = ArgAction::SetTrue,
        help = "Generate .doc from C source and a captured run screenshot"
    )]
    pub auto_doc: bool,

    #[arg(
        long = "run-command",
        help = "Shell command to run the program (runs as-is via sh/powershell)"
    )]
    pub run_command: Option<String>,

    #[arg(
        long = "run-display-template",
        help = "Template for the displayed run path in evidence"
    )]
    pub run_display_template: Option<String>,

    #[arg(
        long = "output-dir",
        short = 'o',
        help = "Output directory for the submission folder and zip"
    )]
    pub output_dir: Option<PathBuf>,

    #[arg(
        long,
        short = 't',
        help = "Screenshot theme (e.g. dracula, monokai, light)"
    )]
    pub theme: Option<String>,

    #[arg(long = "no-watermark", action = ArgAction::SetTrue, help = "Omit the watermark from the generated doc")]
    pub no_watermark: bool,

    #[arg(long, short = 'f', action = ArgAction::SetTrue, help = "Overwrite existing output")]
    pub force: bool,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init,
    Config {
        #[command(subcommand)]
        command: Option<ConfigCommand>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    Show,
    Path,
    Set(ConfigSetArgs),
    Reset,
    Editor,
}

#[derive(Debug, Args)]
pub struct ConfigSetArgs {
    #[arg(long, help = "Default student name")]
    pub name: Option<String>,

    #[arg(long = "id", help = "Default student ID")]
    pub student_id: Option<String>,

    #[arg(long = "output-dir", help = "Default output directory")]
    pub output_dir: Option<PathBuf>,

    #[arg(long = "auto-doc", help = "Default auto-doc behavior (true/false)")]
    pub auto_doc: Option<bool>,

    #[arg(long = "run-command", conflicts_with = "clear_run_command")]
    pub run_command: Option<String>,

    #[arg(long = "clear-run-command", action = ArgAction::SetTrue)]
    pub clear_run_command: bool,

    #[arg(
        long = "run-display-template",
        conflicts_with = "clear_run_display_template"
    )]
    pub run_display_template: Option<String>,

    #[arg(long = "clear-run-display-template", action = ArgAction::SetTrue)]
    pub clear_run_display_template: bool,

    #[arg(long = "theme", conflicts_with = "clear_theme")]
    pub theme: Option<String>,

    #[arg(long = "clear-theme", action = ArgAction::SetTrue)]
    pub clear_theme: bool,

    #[arg(long, conflicts_with = "clear_editor")]
    pub editor: Option<String>,

    #[arg(long, conflicts_with = "editor")]
    pub clear_editor: bool,

    #[arg(
        long = "watermark",
        help = "Show watermark in generated doc (true/false)"
    )]
    pub watermark: Option<bool>,
}
