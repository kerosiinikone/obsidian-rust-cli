use std::{env, fs, path::PathBuf};

use anyhow::Result;

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
            // TODO: Read cfg file / default to a "default_location"
            env::current_dir()?
        };

        if !cfg.is_valid_vault()? {
            return Err(anyhow::Error::msg("Invalid vault"));
        }

        cfg.template.path = if let Some(templ) = &template {
            templ.to_path_buf()
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

        Ok(cfg)
    }

    fn is_valid_vault(&mut self) -> Result<bool> {
        let mut paths = fs::read_dir(&self.vault.as_path()).unwrap();
        Ok(self.vault.is_dir()
            && paths.any(|path_result| path_result.unwrap().file_name() == ".obsidian"))
    }
}
