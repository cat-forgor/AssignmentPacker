mod cli;
mod terminal;
mod config;
mod error;
mod fs;
mod pack;
mod render;
mod ui;
mod update;
mod validate;

use clap::Parser;
use cli::{Cli, Commands};
use owo_colors::OwoColorize;

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e}", "error:".red().bold());
        std::process::exit(1);
    }
}

fn run() -> error::Result<()> {
    let cli = Cli::parse();

    if cli.command.is_some() {
        let has_pack_flags = cli.assignment.is_some()
            || cli.name.is_some()
            || cli.student_id.is_some()
            || cli.c_file.is_some()
            || cli.doc_file.is_some()
            || cli.auto_doc
            || cli.run_command.is_some()
            || cli.input.is_some()
            || cli.timeout.is_some()
            || cli.run_display_template.is_some()
            || cli.output_dir.is_some()
            || cli.theme.is_some()
            || cli.no_watermark
            || cli.force;
        if has_pack_flags {
            return Err(error::Error::Validation(
                "pack flags (like -a, -n, --auto-doc) cannot be used with subcommands".into(),
            ));
        }
    }

    match cli.command {
        Some(Commands::Init) => config::commands::run_init(),
        Some(Commands::Config { command }) => config::commands::run_config_command(command),
        Some(Commands::Update) => update::run(),
        Some(Commands::Themes) => render::theme::run_list(),
        None => pack::run_pack(cli),
    }
}
