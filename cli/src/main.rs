use anyhow::{Ok, Result};
use chrono::Local;
use clap::{Parser, Subcommand};
use cli_core::{config::Config, note::Note, template::TemplArgs, vault::VaultStats};
use std::{
    fs::{self, File},
    io::{self, Read},
    path::PathBuf,
};
use tokio::main;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
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
enum Command {
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
        note: PathBuf,
        idea: String,
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
        note: PathBuf,
    },

    /// Print statistics of the vault
    Stats {},
}

// Make a main struct to hold the exec functions -> core
#[main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let cfg = Config::build(args.vault, args.template)?;

    match args.command {
        Command::New { idea } => exec_new_note(idea, &cfg),
        Command::Append { note, idea } => exec_append_note(idea, note, &cfg),
        Command::Open {} => exec_open_daily(&cfg),
        Command::Show { note } => exec_show_note(note, &cfg),
        Command::Stats {} => exec_vault_stats(&cfg).await,
        _ => Err(anyhow::Error::msg("Invalid command")), // Automatically does this -> clap?
    }
}

async fn exec_vault_stats(cfg: &Config) -> Result<()> {
    let mut stats = VaultStats::default();
    stats.walk_vault(cfg).await?;

    println!("Vault Links: {}", stats.total_link_count);
    println!("Vault Words: {}", stats.total_word_count);
    println!("Most Frequent Tags:");

    for (tag, count) in stats.frequent_tags(3) {
        println!("    {}: {}", tag, count);
    }

    return Ok(());
}

fn exec_new_note(idea: Option<String>, cfg: &Config) -> Result<()> {
    let mut note_path: PathBuf = cfg.vault.clone();

    if let Some(idea) = idea {
        let formatted = format!("{}", Local::now().format("%Y_%m_%d_%H_%M_%S"));
        note_path.push(format!("Note_{}.md", formatted));

        let handle = File::create(note_path.as_path())?;

        let body = cfg.template.render(&TemplArgs {
            body: idea,
            date: formatted,
        })?;

        let mut note = Note::new(&handle, &note_path, Some(body));
        note.write_file_handle()?;

        println!("Created note: {}", note);
        Ok(())
    } else {
        // Prompt for the idea -> make better later
        let mut idea_buffer = String::new();
        while idea_buffer.trim_ascii().is_empty() {
            idea_buffer.clear();
            println!("Please enter your idea (end with Ctrl-D):");
            io::stdin().read_line(&mut idea_buffer)?;
        }
        return exec_new_note(Some(idea_buffer), cfg);
    }
}

fn exec_append_note(idea: String, note: PathBuf, cfg: &Config) -> Result<()> {
    let abs_path = cfg.get_full_path(&note)?;
    let handle = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&abs_path.as_path())?;

    let mut note = Note::new(&handle, &abs_path, None);
    note.append(&idea)?;

    println!("Appended to note: {}", note);
    Ok(())
}

fn exec_open_daily(cfg: &Config) -> Result<()> {
    let mut note_path: PathBuf = cfg.vault.clone();
    let formatted = format!("{}", Local::now().format("%Y-%m-%d"));
    note_path.push(format!("{}.md", formatted));

    let vault_name = cfg
        .vault
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    Note::open(note_path, &vault_name, formatted)?;
    Ok(())
}

fn exec_show_note(note_path: PathBuf, cfg: &Config) -> Result<()> {
    let abs_path = cfg.get_full_path(&note_path)?;
    let mut handle = File::open(&abs_path.as_path())?;
    // let mut buf: Vec<u8> = Vec::new();
    let mut buf = String::new();

    // handle.read_to_end(&mut buf)?;
    // io::stdout().write_all(&buf)?;

    // TODO: parse highlights trick

    handle.read_to_string(&mut buf)?;
    termimad::print_text(&buf);
    Ok(())
}
