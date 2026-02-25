mod cli;
mod compiler;
mod config;
mod error;
mod fs;
mod pack;
mod rtf;
mod screenshot;
mod theme;
mod ui;
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
    match cli.command {
        Some(Commands::Init) => config::run_init(),
        Some(Commands::Config { command }) => config::run_config_command(command),
        None => pack::run_pack(cli),
    }
}
