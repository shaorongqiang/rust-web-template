use std::{
    fs::{File, create_dir_all},
    path::{Path, PathBuf},
};

use anyhow::{Result, anyhow};
use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser, Subcommand};
use clap_complete::{Shell, generate};

use application::{
    Application, CONFIG_FILE_NAME, Config, current_exe_name, init_tracing, new_webstate,
};

#[derive(Parser, Debug)]
pub struct CmdArgs {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    Run {
        #[arg(long, short = 'c', default_value = CONFIG_FILE_NAME)]
        config: String,
        #[command(flatten)]
        args: Config,
    },
    Configure {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Completions {
        #[arg(value_enum)]
        shell: Shell,
        #[arg(long, short = 'o')]
        output: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
pub enum ConfigCommand {
    Generate {
        #[arg(long, short = 'o', default_value = CONFIG_FILE_NAME)]
        output: String,
        #[arg(long, short = 'f')]
        force: bool,
        #[command(flatten)]
        args: Config,
    },
    GenerateKeys {
        #[arg(long, short = 'c', default_value = CONFIG_FILE_NAME)]
        config: String,
    },
    DatabaseInit {
        #[arg(long, short = 'c', default_value = CONFIG_FILE_NAME)]
        config: String,
    },
}

impl CmdArgs {
    pub async fn execute(self, matches: &ArgMatches) -> Result<()> {
        match self.command {
            Command::Run { config, args } => {
                let cfg_path = PathBuf::from(config);
                let run_matches = matches
                    .subcommand_matches("run")
                    .ok_or_else(|| anyhow::anyhow!("run command matches not found"))?;
                Self::run(cfg_path, args, run_matches).await
            }

            Command::Configure { command } => match command {
                ConfigCommand::Generate {
                    output,
                    force,
                    args,
                } => Self::generate_config(PathBuf::from(output), force, args),
                ConfigCommand::GenerateKeys { config } => {
                    Self::generate_keys(PathBuf::from(config))
                }
                ConfigCommand::DatabaseInit { config } => {
                    Self::init_database(PathBuf::from(config)).await
                }
            },

            Command::Completions { shell, output } => Self::write_completions(shell, &output),
        }
    }

    fn write_completions(shell: Shell, output: &Path) -> Result<()> {
        if let Some(parent) = output
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            create_dir_all(parent)?;
        }

        let mut file = File::create(output)?;
        let mut command = Self::command();
        let bin_name = command.get_name().to_string();
        generate(shell, &mut command, bin_name, &mut file);
        eprintln!("Completion script generated: {}", output.display());
        Ok(())
    }

    async fn run(cfg_path: PathBuf, args: Config, run_matches: &ArgMatches) -> Result<()> {
        let exe_name =
            current_exe_name().ok_or_else(|| anyhow::anyhow!("current exe not found"))?;

        if !cfg_path.exists() {
            return Err(anyhow!(
                "config file not found: {} (run `{} config generate --output {}` first)",
                cfg_path.display(),
                exe_name,
                cfg_path.display()
            ));
        }

        let mut cfg = Config::load_from_file(&cfg_path)?;

        cfg.log.merge_with_args(run_matches, args.log);
        cfg.token.merge_with_args(run_matches, args.token);
        cfg.server.merge_with_args(run_matches, args.server);
        cfg.db.merge_with_args(run_matches, args.db);
        cfg.verification
            .merge_with_args(run_matches, args.verification);

        cfg.validate()?;

        let _tracing_guard = init_tracing(&cfg.log, &exe_name)?;
        let state = new_webstate(&cfg.token)?;
        let application = Application::new(state);
        application.run_background().await?;
        application.run(&cfg.server).await
    }

    fn generate_config(cfg_path: PathBuf, force: bool, args: Config) -> Result<()> {
        if cfg_path.exists() && !force {
            return Err(anyhow!(
                "config file already exists: {} (use --force to overwrite)",
                cfg_path.display()
            ));
        }

        let mut cfg = Config::from(args);
        cfg.create_file(&cfg_path)?;
        eprintln!("Config file generated: {}", cfg_path.display());
        Ok(())
    }

    fn generate_keys(cfg_path: PathBuf) -> Result<()> {
        let exe_name =
            current_exe_name().ok_or_else(|| anyhow::anyhow!("current exe not found"))?;
        if !cfg_path.exists() {
            return Err(anyhow!(
                "config file not found: {} (run `{} config generate --output {}` first)",
                cfg_path.display(),
                exe_name,
                cfg_path.display()
            ));
        }
        Config::load_from_file(&cfg_path).and_then(|cfg| cfg.token.generate_keys())?;

        eprintln!(
            "token key files generated from config: {}",
            cfg_path.display()
        );
        Ok(())
    }

    async fn init_database(cfg_path: PathBuf) -> Result<()> {
        let exe_name =
            current_exe_name().ok_or_else(|| anyhow::anyhow!("current exe not found"))?;
        if !cfg_path.exists() {
            return Err(anyhow!(
                "config file not found: {} (run `{} config generate --output {}` first)",
                cfg_path.display(),
                exe_name,
                cfg_path.display()
            ));
        }

        let cfg = Config::load_from_file(&cfg_path)?;
        let _tracing_guard = init_tracing(&cfg.log, &exe_name)?;
        Application::new(new_webstate(&cfg.token)?)
            .init_database()
            .await?;
        tracing::info!(
            "Database initialization completed from config: {}",
            cfg_path.display()
        );
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let matches = CmdArgs::command().get_matches();
    let args = CmdArgs::from_arg_matches(&matches)?;
    args.execute(&matches).await
}
