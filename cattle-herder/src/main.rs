use std::path::PathBuf;
use std::process::ExitCode;

use anyhow::{bail, Context, Result};
use cattle_common::Mode;
use clap::Parser;
use serde::{Deserialize, Serialize};

pub const VERSION: &str = concat!(
    "v",
    env!("CARGO_PKG_VERSION"),
    "-",
    env!("VERGEN_GIT_DESCRIBE"),
    " ",
    env!("VERGEN_BUILD_DATE")
);

/// Cattle Herder
///
/// The stats collection side of the Cattle Herder project
#[derive(Parser, Debug, Clone, PartialEq)]
#[command(author, about, version = VERSION)]
pub struct Args {
    pub config: PathBuf,
}

/// Cattle: We either listen to get polled, or send data to the Herder.
#[derive(Debug, Deserialize, Serialize)]
pub struct CattleConfig {
    /// Herd config: push or pull?
    pub herd_config: cattle_common::Config,
}

fn main() -> Result<ExitCode> {
    let args = Args::parse();
    let config = std::fs::read_to_string(&args.config)
        .context(format!("failed to read config file {:?}", args.config))?;
    let config: CattleConfig = toml::from_str(&config).context(format!(
        "failed to parse Toml config file {:?}",
        args.config
    ))?;

    if matches!(config.herd_config.mode, Mode::Poll(_)) {
        bail!("Error: this is the Cattle application, Poll mode not acceptable.");
    }

    Ok(ExitCode::SUCCESS)
}
