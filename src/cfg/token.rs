use std::{
    fs::{OpenOptions, create_dir_all},
    io::{self, ErrorKind, Write},
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use clap::{ArgMatches, Args};

use jwt_simple::prelude::Ed25519KeyPair;
use serde::{Deserialize, Serialize};

use super::merge_if_overridden;

const TOKEN_ACCESS_KEY_PATH: &str = "cfg/access_key.pem";
const TOKEN_ACCESS_TOKEN_EXPIRED_MINUTES: &str = "10";
const TOKEN_REFRESH_KEY_PATH: &str = "cfg/refresh_key.pem";
const TOKEN_REFRESH_TOKEN_EXPIRED_MINUTES: &str = "120";
const TOKEN_REFRESH_TOKEN_MAX_EXPIRED_MINUTES: &str = "1440";

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
#[command(next_help_heading = "Token Options")]
pub struct TokenConfig {
    #[arg(
        env = "TOKEN_ACCESS_KEY_PATH",
        long = "token-access-key-path",
        default_value = TOKEN_ACCESS_KEY_PATH
    )]
    pub access_key_path: PathBuf,
    #[arg(
        env = "TOKEN_ACCESS_TOKEN_EXPIRED_MINUTES",
        long = "token-access-token-expired-minutes",
        default_value = TOKEN_ACCESS_TOKEN_EXPIRED_MINUTES
    )]
    pub access_token_expired_minutes: u64,

    #[arg(
        env = "TOKEN_REFRESH_KEY_PATH",
        long = "token-refresh-key-path",
        default_value = TOKEN_REFRESH_KEY_PATH
    )]
    pub refresh_key_path: PathBuf,
    #[arg(
        env = "TOKEN_REFRESH_TOKEN_EXPIRED_MINUTES",
        long = "token-refresh-token-expired-minutes",
        default_value = TOKEN_REFRESH_TOKEN_EXPIRED_MINUTES
    )]
    pub refresh_token_expired_minutes: u64,
    #[arg(
        env = "TOKEN_REFRESH_TOKEN_MAX_EXPIRED_MINUTES",
        long = "token-refresh-token-max-expired-minutes",
        default_value = TOKEN_REFRESH_TOKEN_MAX_EXPIRED_MINUTES
    )]
    pub refresh_token_max_expired_minutes: u64,
}

impl TokenConfig {
    pub fn merge_with_args(&mut self, matches: &ArgMatches, arg: Self) {
        merge_if_overridden(
            matches,
            "access_key_path",
            &mut self.access_key_path,
            arg.access_key_path,
        );
        merge_if_overridden(
            matches,
            "access_token_expired_minutes",
            &mut self.access_token_expired_minutes,
            arg.access_token_expired_minutes,
        );
        merge_if_overridden(
            matches,
            "refresh_key_path",
            &mut self.refresh_key_path,
            arg.refresh_key_path,
        );
        merge_if_overridden(
            matches,
            "refresh_token_expired_minutes",
            &mut self.refresh_token_expired_minutes,
            arg.refresh_token_expired_minutes,
        );
        merge_if_overridden(
            matches,
            "refresh_token_max_expired_minutes",
            &mut self.refresh_token_max_expired_minutes,
            arg.refresh_token_max_expired_minutes,
        );
    }

    pub fn validate(&self) -> Result<()> {
        if self.access_key_path.as_os_str().is_empty() {
            Err(anyhow!("token access_key_path must not be empty"))
        } else if self.refresh_key_path.as_os_str().is_empty() {
            Err(anyhow!("token refresh_key_path must not be empty"))
        } else if self.access_key_path == self.refresh_key_path {
            Err(anyhow!(
                "token access_key_path and refresh_key_path must be different"
            ))
        } else if self.access_token_expired_minutes == 0 {
            Err(anyhow!(
                "token access_token_expired_minutes must be greater than 0"
            ))
        } else if self.refresh_token_expired_minutes == 0 {
            Err(anyhow!(
                "token refresh_token_expired_minutes must be greater than 0"
            ))
        } else if self.refresh_token_max_expired_minutes < self.refresh_token_expired_minutes {
            Err(anyhow!(
                "token refresh_token_max_expired_minutes must be greater than or equal to refresh_token_expired_minutes"
            ))
        } else {
            Ok(())
        }
    }

    pub fn generate_keys(&self) -> Result<()> {
        write_key_if_missing(&self.access_key_path)?;
        write_key_if_missing(&self.refresh_key_path)?;
        Ok(())
    }
}

fn generate_key_pem() -> Result<String> {
    Ok(Ed25519KeyPair::generate().to_pem())
}

fn write_new_key(path: &Path, content: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

fn write_key_if_missing(path: &Path) -> Result<()> {
    if !path.exists() {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            create_dir_all(parent)?;
        }
        let key_pem = generate_key_pem()?;
        match write_new_key(path, &key_pem) {
            Ok(()) => {}
            Err(err)
                if err
                    .downcast_ref::<io::Error>()
                    .is_some_and(|err| err.kind() == ErrorKind::AlreadyExists) => {}
            Err(err) => return Err(err),
        }
    }
    Ok(())
}
