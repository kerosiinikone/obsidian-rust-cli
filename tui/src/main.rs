use clap::Parser;
use cli_core::config::Config;
use color_eyre::Result;
use std::{path::PathBuf, process::exit};

use crate::app::App;

mod app;
mod input;
mod new;
mod show;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to vault
    #[arg(short, long)]
    pub vault: Option<PathBuf>,

    /// Path to template
    #[arg(short, long)]
    pub template: Option<PathBuf>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let cfg = Config::build(args.vault, args.template).unwrap_or_else(|_| exit(1));

    color_eyre::install()?;
    let terminal = ratatui::init();
    let app = App::new(cfg).run(terminal);
    ratatui::restore();
    app
}
