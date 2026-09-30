use std::path::PathBuf;

use clap::{Args, Parser};
use gateway::ConfigSource;

mod cmd;

#[derive(Args, Debug, Clone)]
pub(crate) struct ConfigArgs {
    #[arg(short, long, value_name = "config")]
    pub(crate) config: Option<String>,

    #[arg(short, long, value_name = "file")]
    pub(crate) file: Option<PathBuf>,
}

#[derive(Args, Clone)]
pub(crate) struct RunArgs {
    #[command(flatten)]
    pub(crate) config: ConfigArgs,
}

#[derive(Args)]
pub(crate) struct StealthArgs {
    pub(crate) config: Option<String>,
}

#[derive(Parser)]
struct Cli {
    #[command(flatten)]
    run: RunArgs,
}

pub fn run() -> anyhow::Result<()> {
    let args = Cli::parse();
    cmd::run::execute(args.run)
}

pub(crate) fn read_config_contents(
    config: &ConfigArgs,
) -> anyhow::Result<(String, Option<ConfigSource>)> {
    match (&config.config, &config.file) {
        (Some(_), Some(_)) => {
            anyhow::bail!("only one of --config or --file may be set")
        }

        (Some(config), None) => Ok((
            config.clone(),
            Some(ConfigSource::Static(config.clone().into())),
        )),

        (None, Some(file)) => {
            let contents = std::fs::read_to_string(file)?;
            Ok((contents, Some(ConfigSource::File(file.clone()))))
        }

        (None, None) => Ok((String::new(), None)),
    }
}