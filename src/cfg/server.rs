use std::net::SocketAddr;

use anyhow::{Result, anyhow};
use clap::{ArgMatches, Args};
use serde::{Deserialize, Serialize};
use url::Url;

use super::merge_if_overridden;

const LISTEN: &str = "0.0.0.0:8580";
const ACCESS_URL: &str = "http://127.0.0.1:8580";

#[derive(Debug, Args, Serialize, Deserialize)]
#[command(next_help_heading = "Server Options")]
pub struct ServerConfig {
    #[arg(
        env = "SERVER_LISTEN",
        short = 'l',
        long = "server-listen",
        default_value = LISTEN
    )]
    pub listen: SocketAddr,
    #[arg(
        env = "SERVER_ACCESS_URL",
        long = "server-access-url",
        default_value = ACCESS_URL
    )]
    pub access_url: Url,
    #[arg(
        env = "SERVER_ALLOWED_ORIGINS",
        long = "server-allowed-origins",
        value_delimiter = ','
    )]
    pub allowed_origins: Vec<Url>,
}

impl ServerConfig {
    pub fn merge_with_args(&mut self, matches: &ArgMatches, arg: Self) {
        merge_if_overridden(matches, "listen", &mut self.listen, arg.listen);
        merge_if_overridden(matches, "access_url", &mut self.access_url, arg.access_url);
        merge_if_overridden(
            matches,
            "allowed_origins",
            &mut self.allowed_origins,
            arg.allowed_origins,
        );
    }

    pub fn validate(&self) -> Result<()> {
        if self.listen.port() == 0 {
            return Err(anyhow!("server listen port must be greater than 0"));
        }

        validate_http_url("server access_url", &self.access_url)?;
        validate_base_url("server access_url", &self.access_url)?;

        for origin in &self.allowed_origins {
            validate_http_url("server allowed origin", origin)?;
            validate_origin_url("server allowed origin", origin)?;
        }

        Ok(())
    }
}

fn validate_http_url(name: &str, url: &Url) -> Result<()> {
    if !matches!(url.scheme(), "http" | "https") {
        Err(anyhow!("{name} must use http or https: {url}"))
    } else if url.host().is_none() {
        Err(anyhow!("{name} must include a host: {url}"))
    } else {
        Ok(())
    }
}

fn validate_base_url(name: &str, url: &Url) -> Result<()> {
    if url.query().is_some() || url.fragment().is_some() {
        Err(anyhow!(
            "{name} must not include a query or fragment: {url}"
        ))
    } else {
        Ok(())
    }
}

fn validate_origin_url(name: &str, url: &Url) -> Result<()> {
    if url.path() != "/" || url.query().is_some() || url.fragment().is_some() {
        Err(anyhow!(
            "{name} must be an origin without path, query, or fragment: {url}"
        ))
    } else {
        Ok(())
    }
}
