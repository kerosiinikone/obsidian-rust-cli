use anyhow::{Ok, Result};
use chrono::prelude::*;
use clap::{Parser, Subcommand};
use cli_core::{
    config::Config,
    template::TemplArgs,
    vault::{Note, TagMap, VaultStats},
};
use once_cell::sync::Lazy;
use regex::Regex;
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};
use tokio::{io::AsyncReadExt, main, task::JoinSet};
use walkdir::{DirEntry, WalkDir};

static LINKS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"\[\[.*?\]\]").unwrap());
static TAGS_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"#\w+").unwrap());

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
        Command::Stats {} => {
            let mut set: JoinSet<Note> = JoinSet::new();
            let mut stats = VaultStats::default();

            for entry in WalkDir::new(cfg.vault.clone())
                .into_iter()
                .filter_entry(|e| !is_hidden(e))
                .filter_map(|e| e.ok())
                .filter(|e| e.path().is_file() && e.path().extension() == Some(OsStr::new("md")))
            {
                set.spawn(async move {
                    let mut contents = String::new();
                    let mut file = tokio::fs::OpenOptions::new()
                        .read(true)
                        .open(entry.path())
                        .await
                        .unwrap();
                    file.read_to_string(&mut contents).await.unwrap();

                    let word_count = contents.split_ascii_whitespace().count();
                    let link_count = LINKS_REGEX.find_iter(&contents).count();
                    let mut tag_counts: TagMap = TagMap::new();

                    for t in TAGS_REGEX.find_iter(&contents) {
                        let tag = t.as_str().to_string();
                        *tag_counts.entry(tag).or_insert(0) += 1;
                    }

                    Note {
                        link_count: link_count,
                        word_count: word_count,
                        tags: tag_counts,
                    }
                });
            }

            while let Some(res) = set.join_next().await {
                stats.merge(res?);
            }

            let mut freq_tags: Vec<(&String, &u32)> = stats.tags.iter().collect();
            freq_tags.sort_by_key(|t| t.1);
            freq_tags.reverse();

            println!("Vault Links: {}", stats.total_link_count);
            println!("Vault Words: {}", stats.total_word_count);
            println!("Most Frequent Tags:");
            for t in freq_tags.iter().take(3) {
                println!("    {}: {}", t.0, t.1);
            }

            return Ok(());
        }
        _ => Err(anyhow::Error::msg("Invalid command")), // Automatically does this -> clap?
    }
}

fn exec_new_note(idea: Option<String>, cfg: &Config) -> Result<()> {
    let mut note_path: PathBuf = cfg.vault.clone();

    if let Some(idea) = idea {
        let local: DateTime<Local> = Local::now();
        let formatted = format!("{}", local.format("%Y_%m_%d_%H_%M_%S"));

        note_path.push(format!("Note_{}.md", &formatted));
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
            println!("Please enter your idea (end with Ctrl-D):");
            io::stdin().read_line(&mut idea_buffer)?;
            idea_buffer.clear();
        }
        return exec_new_note(Some(idea_buffer), cfg);
    }

    println!("Created note: {}", note_path.display());
    Ok(())
}

fn exec_append_note(idea: String, note: PathBuf, cfg: &Config) -> Result<()> {
    let abs_path = cfg.get_full_path(&note)?;

    let mut note_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&abs_path.as_path())?;

    note_file.write(b"\n")?;
    note_file.write_all(&idea.as_bytes())?;

    println!("Appended to note: {}", abs_path.display());
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

    let vault_name = cfg
        .vault
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");

    let obs_link = format!(
        "obsidian://open?vault={}&file={}",
        urlencoding::encode(vault_name),
        urlencoding::encode(&formatted)
    );

    open::that(obs_link)?;

    Ok(())
}

fn exec_show_note(note_path: PathBuf, cfg: &Config) -> Result<()> {
    let abs_path = cfg.get_full_path(&note_path)?;
    let mut handle = File::open(&abs_path.as_path())?;
    let mut buf: Vec<u8> = Vec::new();

    handle.read_to_end(&mut buf)?;
    io::stdout().write_all(&buf)?;

    Ok(())
}

fn is_hidden(entry: &DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map(|s| s.starts_with("."))
        .unwrap_or(false)
}
