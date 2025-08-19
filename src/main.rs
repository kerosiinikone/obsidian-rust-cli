use anyhow::{Ok, Result};
use chrono::prelude::*;
use clap::Parser;
use obsidian_rust_cli::{config::Config, template::TemplArgs};
use std::{
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

use clap::Subcommand;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

    /// Path to vault
    #[arg(short, long)]
    pub vault: Option<PathBuf>,

    /// Path to template
    #[arg(short, long)]
    pub template: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
pub enum Command {
    /// Create a new note from an idea
    New { idea: Option<String> },

    /// Edit the config file (todo!)
    Config {
        #[arg(short, long)]
        vault: Option<PathBuf>,
        #[arg(short, long)]
        template: Option<PathBuf>,
    },

    /// Append to an existing note. If not exists, exec "new"
    Append {
        #[arg(short)]
        note: Option<PathBuf>,
        idea: Option<String>,
    },

    /// Open the daily(?) note -> could also be just "Daily"
    ///
    /// Example: obsidian://open?vault=TestVault&file=Test%20Note.
    /// Has to also format the date according to Obsidian's "daily".
    /// Format as default: "2025-08-15"; "YYYY-MM-DD"
    Open {},

    /// Pretty print a note with formatting
    Show {
        #[arg(short)]
        note: Option<PathBuf>,
    },
}

// Print statistics of the vault
// Status {},

// Make a main struct to hold the exec functions?
fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let cfg = Config::build(args.vault, args.template)?;

    match args.command {
        Command::New { idea } => exec_new_note(idea, &cfg),
        Command::Append { note, idea } => exec_append_note(idea, note, &cfg),
        Command::Open {} => exec_open_daily(&cfg),
        Command::Show { note } => exec_show_note(note, &cfg),
        _ => Err(anyhow::Error::msg("Invalid command")), // Automatically does this -> clap?
    }
}

fn exec_new_note(idea: Option<String>, cfg: &Config) -> Result<()> {
    if let Some(idea) = idea {
        let mut note_path: PathBuf = cfg.vault.clone();

        let local: DateTime<Local> = Local::now();
        let formatted = format!("{}", local.format("%Y_%m_%d_%H_%M_%S"));

        note_path.push(format!("Note_{}.md", formatted));
        let mut handle = File::create(note_path.as_path())?;

        let body = cfg.template.render(&TemplArgs {
            body: idea,
            date: formatted,
        })?;

        handle.write_all(body.as_bytes())?;
    } else {
        // Prompt for the idea -> make better later
        let mut idea_buffer = String::new();
        while idea_buffer.trim_ascii().is_empty() {
            io::stdin().read_line(&mut idea_buffer)?;
            idea_buffer.clear();
        }
        return exec_new_note(Some(idea_buffer), cfg);
    }

    Ok(())
}

fn exec_append_note(idea: Option<String>, note: Option<PathBuf>, cfg: &Config) -> Result<()> {
    let note_path: PathBuf = note.expect("Expected a valid note path");

    let mut abs_path = cfg.vault.clone();
    abs_path.push(&note_path);

    if !abs_path.is_file() {
        return Err(anyhow::Error::msg("Invalid note path"));
    }

    let mut note_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&abs_path.as_path())?;

    note_file.write(b"\n")?;
    note_file.write_all(&idea.unwrap().as_bytes())?;

    Ok(())
}

fn exec_open_daily(cfg: &Config) -> Result<()> {
    let mut note_path: PathBuf = cfg.vault.clone();

    let local: DateTime<Local> = Local::now();
    let formatted = format!("{}", local.format("%Y-%m-%d"));

    note_path.push(format!("{}.md", formatted));

    fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(note_path.as_path())?;

    let vault_name = cfg.vault.iter().last().unwrap().to_str().unwrap();
    let obs_link = format!("obsidian://open?vault={}&file={}", vault_name, formatted);

    open::that(obs_link)?;

    Ok(())
}

fn exec_show_note(note: Option<PathBuf>, cfg: &Config) -> Result<()> {
    let note_path: PathBuf = note.expect("Expected a valid note path");

    let mut abs_path = cfg.vault.clone();
    abs_path.push(&note_path);

    if !abs_path.is_file() {
        return Err(anyhow::Error::msg("Invalid note path"));
    }

    let mut handle = File::open(&abs_path.as_path())?;
    let mut buf: Vec<u8> = Vec::new();

    handle.read_to_end(&mut buf)?;
    io::stdout().write_all(&buf)?;

    Ok(())
}
