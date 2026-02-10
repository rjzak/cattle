use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use cattle_common::Mode;
use clap::Parser;
use serde::{Deserialize, Serialize};

pub const VERSION: &str = concat!(env!("CATTLE_VERSION"), " ", env!("CATTLE_BUILD_DATE"));

/// Cattle Monitor
///
/// The reporting side of the Cattle Monitor project
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, about, version = VERSION)]
pub struct Args {
    pub config: PathBuf,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HerderConfig {
    /// Herd config: pull or poll?
    pub herd_config: cattle_common::Config,

    /// Web interface port for displaying information
    pub web_port: u16,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();
    let config = std::fs::read_to_string(&args.config)
        .context(format!("failed to read config file {:?}", args.config))?;
    let config: HerderConfig = toml::from_str(&config).context(format!(
        "failed to parse Toml config file {:?}",
        args.config
    ))?;

    if matches!(config.herd_config.mode, Mode::Push(_)) {
        bail!("Error: this is the Herder application, Push mode not acceptable.");
    }

    Ok(ExitCode::SUCCESS)
}
