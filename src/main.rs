use anyhow::{Ok, Result};
use chrono::prelude::*;
use clap::{Parser, Subcommand};
use std::{
    env,
    fs::File,
    io::{Read, Write},
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

impl Template {
    fn parse_string(&mut self) -> Result<()> {
        // Assume that the path to template is valid
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
    Append {},

    /// Open the daily(?) note
    Open {},

    /// Pretty print a note
    Show {},
}
// Print statistics of the vault
// Status {},

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Parse config
    let mut cfg = Config::default();
    // Order: flags, env, cfg file, traversion?
    cfg.vault = if let Some(vault) = args.vault {
        vault
    } else if let Result::Ok(vault) = env::var("VAULT_PATH") {
        PathBuf::from(vault)
    } else {
        // TODO: Read cfg file / default to a "default_location"
        env::current_dir()?
    };

    // Validate the vault location (TODO: has to have .obsidian also)
    if !cfg.vault.is_dir() {
        return Err(anyhow::Error::msg("Invalid vault"));
    }

    // Template parsing, etc...
    cfg.template.path = if let Some(templ) = args.template {
        templ
    } else {
        let mut curr = env::current_dir()?;
        curr.push("config");
        curr.push("default_template.md");
        curr
    };

    if !cfg.template.path.is_file() {
        return Err(anyhow::Error::msg("Invalid templ path"));
    }

    cfg.template.parse_string()?;

    // eprintln!("Template: {:?}", cfg.template.template);

    match args.command {
        Command::New { idea } => {
            // eprintln!("Idea: {idea}");
            // eprintln!("Vault: {:?}", cfg.vault);

            // Create a new .md file (with a template, if exists -> parsed earlier)
            let mut note_path: PathBuf = cfg.vault.clone();

            let local: DateTime<Local> = Local::now();
            let formatted = format!("{}", local.format("%Y_%m_%d_%H_%M_%S"));

            note_path.push(format!("Note_{}.md", formatted));
            let mut handle = File::create(note_path.as_path())?;

            let mut new_note_templ = cfg.template.template.clone();

            // Refactor into a more elegant function ("format", etc)
            new_note_templ = new_note_templ.replace("?time", &formatted);

            if let Some(idea) = idea {
                // Generate path
                new_note_templ = new_note_templ.replace("?body", &idea);
                handle.write_all(new_note_templ.as_bytes())?;
            }

            // Prompt to give the idea?
        }
        _ => return Ok(()),
    }
    Ok(())
}
