use anyhow::Result;
use clap::{ArgMatches, Args, parser::ValueSource};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Args, Serialize, Deserialize)]
#[command(next_help_heading = "Verification Options")]
pub struct VerificationConfig {
    #[arg(
        env = "VERIFICATION_CODE_EXPIRED_MINUTES",
        long = "verification-code-expired-minutes",
        default_value = "5",
        help = "Verification code lifetime in minutes. Example: 5"
    )]
    pub code_expired_minutes: u64,
}

impl VerificationConfig {
    pub fn code_expired_seconds(&self) -> i64 {
        (self.code_expired_minutes * 60) as i64
    }

    pub fn merge_with_args(&mut self, matches: &ArgMatches, arg: Self) {
        match matches.value_source("code_expired_minutes") {
            Some(ValueSource::CommandLine) | Some(ValueSource::EnvVariable) => {
                self.code_expired_minutes = arg.code_expired_minutes;
            }
            _ => {}
        }
    }

    pub fn validate(&self) -> Result<()> {
        Ok(())
    }
}
