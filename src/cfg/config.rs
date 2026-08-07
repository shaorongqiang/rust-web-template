use std::{fs, io::Write, path::Path};

use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use tempfile::NamedTempFile;

use super::{DatabaseConfig, LogConfig, ServerConfig, TokenConfig, VerificationConfig};

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct Config {
    #[command(flatten)]
    pub log: LogConfig,
    #[command(flatten)]
    pub server: ServerConfig,
    #[command(flatten)]
    pub token: TokenConfig,
    #[command(flatten)]
    pub db: DatabaseConfig,
    #[command(flatten)]
    pub verification: VerificationConfig,
}

impl Config {
    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let cfg = toml::from_str::<Self>(&content)?;

        cfg.validate()?;
        Ok(cfg)
    }

    pub fn create_file(&mut self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        self.validate()?;
        let content = toml::to_string_pretty(self)?;
        let cfg_dir = config_dir(path);

        if !cfg_dir.exists() {
            fs::create_dir_all(cfg_dir)?;
        }
        write_file_atomically(path, &content)?;

        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        self.log.validate()?;
        self.server.validate()?;
        self.db.validate()?;
        self.token.validate()?;
        self.verification.validate()?;
        Ok(())
    }
}

fn config_dir(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

fn write_file_atomically(path: &Path, content: &str) -> Result<()> {
    let parent = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut temp_file = NamedTempFile::new_in(parent)?;
    temp_file.write_all(content.as_bytes())?;
    temp_file.flush()?;
    temp_file.persist(path)?;
    Ok(())
}
