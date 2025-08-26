use anyhow::Result;
use std::{
    env,
    fs::{self, File},
    io::Read,
    path::PathBuf,
};
use toml::Table;

use crate::template::Template;

#[derive(Debug, Clone, Default)]
pub struct Config {
    /// Target vault location
    pub vault: PathBuf,
    /// Templating for new idea notes
    pub template: Template,
}

impl Config {
    pub fn build(vault: Option<PathBuf>, template: Option<PathBuf>) -> Result<Self> {
        let mut cfg = Config::default();

        cfg.vault = if let Some(vault) = &vault {
            vault.to_path_buf()
        } else if let Result::Ok(vault) = env::var("VAULT_PATH") {
            PathBuf::from(vault)
        } else {
            let mut path = env::current_dir()?;
            path.push("config");
            path.push("default.toml");
            Self::parse_cfg_file(path)?
        };

        if !cfg.is_valid_vault()? {
            return Err(anyhow::Error::msg("Invalid vault"));
        }

        cfg.template.path = if let Some(templ) = &template {
            templ.to_path_buf()
        } else {
            let mut path = env::current_dir()?;
            path.push("config");
            path.push("default_template.md");
            path
        };

        if !cfg.template.path.is_file() {
            return Err(anyhow::Error::msg("Invalid templ path"));
        }
        cfg.template.parse_string()?;

        Ok(cfg)
    }

    pub fn get_full_path(&self, note_path: &PathBuf) -> Result<PathBuf> {
        let mut abs_path = self.vault.clone();
        abs_path.push(&note_path);
        if !abs_path.is_file() {
            return Err(anyhow::Error::msg("Invalid note path"));
        }
        Ok(abs_path.to_path_buf())
    }

    fn is_valid_vault(&mut self) -> Result<bool> {
        let mut paths = fs::read_dir(&self.vault.as_path())?;
        Ok(self.vault.is_dir()
            && paths.any(|path_result| path_result.unwrap().file_name() == ".obsidian"))
    }

    fn parse_cfg_file(path: PathBuf) -> Result<PathBuf> {
        let mut file = File::open(path.as_path())?;
        let mut buf: String = String::new();
        file.read_to_string(&mut buf)?;

        let vault_path = buf.parse::<Table>()?;
        let vault_path = vault_path["vault_path"].as_str().unwrap_or("");

        Ok(PathBuf::from(vault_path))
    }
}
