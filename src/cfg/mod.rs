mod log;
pub use log::LogConfig;

mod token;
pub use token::TokenConfig;

mod server;
pub use server::ServerConfig;

mod database;
pub use database::DatabaseConfig;

mod verification;
pub use verification::VerificationConfig;

mod config;
pub use config::Config;

pub const CONFIG_FILE_NAME: &str = "cfg/config.toml";

use clap::{ArgMatches, parser::ValueSource};

pub(super) fn merge_if_overridden<T>(matches: &ArgMatches, id: &str, target: &mut T, value: T) {
    match matches.value_source(id) {
        Some(ValueSource::CommandLine) | Some(ValueSource::EnvVariable) => {
            *target = value;
        }
        _ => {}
    }
}
