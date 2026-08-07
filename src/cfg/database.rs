use anyhow::{Result, anyhow};
use clap::{ArgMatches, Args};
use serde::{Deserialize, Serialize};
use url::Url;

use super::merge_if_overridden;

const URL: &str = "postgres://app:change-me@127.0.0.1:5432/app";

#[derive(Debug, Args, Serialize, Deserialize)]
#[command(next_help_heading = "Database Options")]
pub struct DatabaseConfig {
    #[arg(
        env = "DATABASE_URL",
        long = "database-url",
        default_value =  URL
    )]
    pub url: Url,
    #[arg(env = "DATABASE_ENABLE_LOGGING", long = "database-enable-logging")]
    pub enable_logging: bool,
    #[arg(env = "DATABASE_MIN_CONNECTIONS", long = "database-min-connections")]
    pub min_connections: Option<u32>,
    #[arg(env = "DATABASE_MAX_CONNECTIONS", long = "database-max-connections")]
    pub max_connections: Option<u32>,
    #[arg(env = "DATABASE_CONNECT_TIMEOUT", long = "database-connect-timeout")]
    pub connect_timeout: Option<u64>,
    #[arg(env = "DATABASE_IDLE_TIMEOUT", long = "database-idle-timeout")]
    pub idle_timeout: Option<u64>,
}

impl DatabaseConfig {
    pub fn merge_with_args(&mut self, matches: &ArgMatches, arg: Self) {
        merge_if_overridden(matches, "url", &mut self.url, arg.url);
        merge_if_overridden(
            matches,
            "enable_logging",
            &mut self.enable_logging,
            arg.enable_logging,
        );
        merge_if_overridden(
            matches,
            "min_connections",
            &mut self.min_connections,
            arg.min_connections,
        );
        merge_if_overridden(
            matches,
            "max_connections",
            &mut self.max_connections,
            arg.max_connections,
        );
        merge_if_overridden(
            matches,
            "connect_timeout",
            &mut self.connect_timeout,
            arg.connect_timeout,
        );
        merge_if_overridden(
            matches,
            "idle_timeout",
            &mut self.idle_timeout,
            arg.idle_timeout,
        );
    }

    pub fn validate(&self) -> Result<()> {
        if !matches!(self.url.scheme(), "postgres" | "postgresql") {
            Err(anyhow!(
                "database url must use postgres or postgresql: {}",
                self.url
            ))
        } else if self.url.host().is_none() {
            Err(anyhow!("database url must include a host: {}", self.url))
        } else if let (Some(min), Some(max)) = (self.min_connections, self.max_connections)
            && min > max
        {
            Err(anyhow!(
                "database min_connections must be less than or equal to max_connections"
            ))
        } else if matches!(self.max_connections, Some(0)) {
            Err(anyhow!("database max_connections must be greater than 0"))
        } else if matches!(self.connect_timeout, Some(0)) {
            Err(anyhow!("database connect_timeout must be greater than 0"))
        } else if matches!(self.idle_timeout, Some(0)) {
            Err(anyhow!("database idle_timeout must be greater than 0"))
        } else {
            Ok(())
        }
    }
}
