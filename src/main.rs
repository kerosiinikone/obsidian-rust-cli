use anyhow::Ok;
use clap::{Parser, Subcommand};
use std::{env, fs::File, io::Write, path::PathBuf};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,

    /// Path to vault
    #[arg(short, long)]
    vault: Option<PathBuf>,
}

#[derive(Debug, Clone)]
struct Config {
    vault: PathBuf,
    // templating -> similar loading scheme?
}

impl Config {
    fn new() -> Self {
        Self {
            vault: PathBuf::new(),
        }
    }
}

#[derive(Subcommand, Debug)]
#[clap(rename_all = "snake_case")]
enum Command {
    /// Create a new note from an idea
    New { idea: String },

    /// Edit the config file (todo!)
    Config {
        #[arg(short, long)]
        vault: Option<PathBuf>,

        #[arg(short, long)]
        template: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Parse config
    let mut cfg = Config::new();
    // Order: flags, env, cfg file, traversion?
    cfg.vault = if let Some(vault) = args.vault {
        vault
    } else if let Result::Ok(vault) = env::var("VAULT_PATH") {
        PathBuf::from(vault)
    } else {
        // TODO: Read cfg file / default to a "default_location"
        env::current_dir()?
    };

    eprintln!("{:?}", cfg.vault);

    // Template parsing, etc...

    // Validate the vault location (has to have .obsidian)
    if !cfg.vault.is_dir() {
        return Err(anyhow::Error::msg("Invalid vault"));
    }

    match args.command {
        Command::New { idea } => {
            // eprintln!("Idea: {idea}");
            // eprintln!("Vault: {:?}", cfg.vault);

            // Create a new .md file (with a template, if exists -> parsed earlier)
            // Name based on timestamp or configured prefixes?

            // Generate path
            let mut note_path: PathBuf = cfg.vault.clone();
            note_path.push("test_file.md");

            // Create file
            let mut handle = File::create(note_path.as_path())?;
            handle.write_all(idea.as_bytes())?;
        }
        _ => return Ok(()),
    }
    Ok(())
}
