use anyhow::{Ok, Result};
use chrono::prelude::*;
use clap::{Parser, Subcommand};
use std::{
    env,
    fs::{self, File},
    io::{self, Read, Write},
    path::PathBuf,
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Path to vault
    #[arg(short, long)]
    vault: Option<PathBuf>,

    /// Path to template
    #[arg(short, long)]
    template: Option<PathBuf>,
}

#[derive(Debug, Clone, Default)]
struct Template {
    path: PathBuf,
    template: String,
}

// Own parser (struct?) to support more intricate
// templates later?
impl Template {
    fn parse_string(&mut self) -> Result<()> {
        let mut file = File::open(self.path.as_path())?;
        file.read_to_string(&mut self.template)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
struct Config {
    /// Target vault location
    vault: PathBuf,
    /// Templating for new idea notes
    template: Template,
}

impl Config {
    fn is_valid_vault(&mut self) -> bool {
        let mut paths = fs::read_dir(&self.vault.as_path()).unwrap();
        self.vault.is_dir() // Might be redundant
            && paths.any(|path_result| path_result.unwrap().file_name() == ".obsidian")
    }
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
        note: Option<PathBuf>,
        idea: Option<String>,
    },

    /// Open the daily(?) note -> could also be just "Daily"
    ///
    /// Example: obsidian://open?vault=TestVault&file=Test%20Note.
    /// Has to also format the date according to Obsidian's "daily".
    /// Format as default: "2025-08-15"; "YYYY-MM-DD"
    Open {},

    /// Pretty print a note
    Show {},
}

// Print statistics of the vault
// Status {},

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let mut cfg = Config::default();
    cfg.vault = if let Some(vault) = args.vault {
        vault
    } else if let Result::Ok(vault) = env::var("VAULT_PATH") {
        PathBuf::from(vault)
    } else {
        // TODO: Read cfg file / default to a "default_location"
        env::current_dir()?
    };

    if !cfg.is_valid_vault() {
        return Err(anyhow::Error::msg("Invalid vault"));
    }

    cfg.template.path = if let Some(templ) = args.template {
        templ
    } else {
        // todo!
        let mut curr = env::current_dir()?;
        curr.push("config");
        curr.push("default_template.md");
        curr
    };

    if !cfg.template.path.is_file() {
        return Err(anyhow::Error::msg("Invalid templ path"));
    }
    cfg.template.parse_string()?;

    match args.command {
        Command::New { idea } => {
            let mut note_path: PathBuf = cfg.vault.clone();

            let local: DateTime<Local> = Local::now();
            let formatted = format!("{}", local.format("%Y_%m_%d_%H_%M_%S"));

            note_path.push(format!("Note_{}.md", formatted));
            let mut handle = File::create(note_path.as_path())?;

            let mut new_note_templ = cfg.template.template.clone();

            // TODO: Refactor into a more elegant function ("format", etc) and
            // check for a list of "?var" for different templating details
            new_note_templ = new_note_templ.replace("?time", &formatted);

            if let Some(idea) = idea {
                // Generate path
                new_note_templ = new_note_templ.replace("?body", &idea);
                handle.write_all(new_note_templ.as_bytes())?;
            } else {
                // Prompt for the idea -> make better later
                let mut idea_buffer = String::new();
                while idea_buffer.trim_ascii().is_empty() {
                    io::stdin().read_line(&mut idea_buffer)?;
                    idea_buffer.clear();
                }
                eprintln!("{:?}", idea_buffer)
            }
        }
        Command::Append { note, idea } => {
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
        }
        Command::Open {} => {
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

            open::that(obs_link)?
        }
        _ => return Ok(()),
    }
    Ok(())
}
