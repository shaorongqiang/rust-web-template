use std::{fs, path::PathBuf};

use anyhow::{Result, anyhow};
use clap::{ArgMatches, Args, ValueEnum};
use serde::{Deserialize, Serialize};

use super::merge_if_overridden;

const LEVEL: &str = "info";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl LogLevel {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "error",
        }
    }
}

#[derive(Debug, Args, Serialize, Deserialize)]
#[command(next_help_heading = "Logging Options")]
pub struct LogConfig {
    #[arg(env = "LOG_DIR", short = 'D', long = "log-dir")]
    pub dir: Option<PathBuf>,
    #[arg(
        env = "LOG_LEVEL",
        short = 'L',
        long = "log-level",
        default_value = LEVEL,
        value_enum
    )]
    pub level: LogLevel,
}

impl LogConfig {
    pub fn merge_with_args(&mut self, matches: &ArgMatches, arg: Self) {
        merge_if_overridden(matches, "dir", &mut self.dir, arg.dir);
        merge_if_overridden(matches, "level", &mut self.level, arg.level);
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(dir) = &self.dir {
            if dir.as_os_str().is_empty() {
                Err(anyhow!("log dir must not be empty"))
            } else if let Ok(metadata) = fs::metadata(dir)
                && !metadata.is_dir()
            {
                Err(anyhow!(
                    "log dir must point to a directory: {}",
                    dir.display()
                ))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}
