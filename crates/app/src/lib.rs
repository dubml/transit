use std::path::{Path, PathBuf};

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
            let file = if file == std::path::Path::new("-") {
                PathBuf::from("/dev/stdin")
            } else {
                file.clone()
            };

            let contents = std::fs::read_to_string(&file)?;

            let source = if is_read_once_path(&file) {
                ConfigSource::Static(contents.clone().into())
            } else {
                ConfigSource::File(file)
            };

            Ok((contents, Some(source)))
        }

        (None, None) => {
            let dir = default_config_dir()?;
            let file = dir.join("config.yaml");
            ensure_default_config_file(&file)?;
            let contents = std::fs::read_to_string(&file)?;
            Ok((contents, Some(ConfigSource::File(file))))
        }
    }
}

fn default_config_dir() -> anyhow::Result<PathBuf> {
    let config_dir = Path::new("/config");
    if existing_writable_dir(config_dir) {
        return Ok(config_dir.to_path_buf());
    }
    if config_dir.exists() {
        anyhow::bail!(
            "{} exists but is not writable; make it writable, pass --file, or pass --config",
            config_dir.display()
        );
    }
    if running_in_container() {
        anyhow::bail!(
            "{} is not mounted; mount a writable {}, pass --file, or pass --config",
            config_dir.display(),
            config_dir.display()
        );
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("HOME is not set; pass --config or --file"))?;
    Ok(home.join(".config").join("transit"))
}

fn running_in_container() -> bool {
    std::env::var("TRANSIT_ENV").is_ok_and(|value| value == "container")
}

fn existing_writable_dir(path: &Path) -> bool {
    if !path.is_dir() {
        return false;
    }
    let probe = path.join(format!(".transit-write-test-{}", std::process::id()));
    match std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&probe)
    {
        Ok(_) => {
            let _ = std::fs::remove_file(probe);
            true
        }
        Err(_) => false,
    }
}

fn ensure_default_config_file(path: &Path) -> anyhow::Result<()> {
    if path.exists() {
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, "# Transit configuration\nstorage:\n  mode: file\n")?;
    Ok(())
}

fn is_read_once_path(path: &std::path::Path) -> bool {
    path == std::path::Path::new("/dev/stdin")
        || path.starts_with("/dev/fd")
        || path.starts_with("/proc/self/fd")
}
