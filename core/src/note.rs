use std::{
    fmt::Display,
    fs::{File, OpenOptions},
    io::Write,
    path::PathBuf,
};

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Note<'a> {
    pub body: Option<String>,
    pub path: &'a PathBuf,
    pub handle: &'a File,
}

impl<'a> Note<'a> {
    pub fn new(handle: &'a File, path: &'a PathBuf, body: Option<String>) -> Self {
        Self {
            body: body,
            handle: handle,
            path: path,
        }
    }

    // This opens the daily note in Obs, another func might open it for reading (aka read it)
    pub fn open(path: PathBuf, vault_name: &str, formatted: String) -> Result<()> {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(path.as_path())?;
        open::that(format!(
            "obsidian://open?vault={}&file={}",
            urlencoding::encode(vault_name),
            urlencoding::encode(&formatted)
        ))?;
        Ok(())
    }

    pub fn append(&mut self, idea: &str) -> Result<()> {
        self.handle.write(b"\n")?;
        self.handle.write_all(&idea.as_bytes())?;
        Ok(())
    }

    pub fn write_file_handle(&mut self) -> Result<()> {
        self.handle
            .write_all(self.body.clone().context("No body to write")?.as_bytes())?;
        Ok(())
    }
}

impl<'a> Display for Note<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path.display())
    }
}
