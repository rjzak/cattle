use cattle_common::{CattleInitialConnect, Mode};

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Parser;
use p384::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};
use uuid::Uuid;

pub const ID_FILE: &str = "/var/cache/cattle-herder/id.txt";

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

#[derive(Debug)]
pub struct CattleState {
    /// Handle on the `sysconfig::System` object for updates without a mutable reference to this object
    pub sys: Arc<RwLock<System>>,

    /// Uuid which uniquely identifies this system to the monitor.
    pub id: Uuid,

    /// Public key
    pub pkey: PublicKey,

    /// Secret key
    skey: SecretKey,
}

impl Default for CattleState {
    fn default() -> Self {
        let uuid_path = Path::new(ID_FILE);

        let uuid = if uuid_path.exists() {
            let uuid = std::fs::read(ID_FILE)
                .unwrap_or_else(|_| panic!("failed to read ID file {ID_FILE}"));
            Uuid::from_slice(&uuid)
                .unwrap_or_else(|_| panic!("failed to parse ID {uuid:?} from file {ID_FILE}"))
        } else {
            let uuid = Uuid::new_v4();
            std::fs::write(ID_FILE, uuid.as_bytes())
                .unwrap_or_else(|_| panic!("failed to save UUID to {ID_FILE}"));
            uuid
        };

        let mut rand = rand::thread_rng();
        let skey = SecretKey::random(&mut rand);
        let pkey = skey.public_key();

        CattleState {
            sys: Arc::new(RwLock::new(System::new_all())),
            id: uuid,
            pkey,
            skey,
        }
    }
}

impl CattleState {
    fn update(&self) {
        if let Ok(mut sys) = self.sys.write() {
            sys.refresh_all();
            sys.refresh_cpu();
        }
    }

    pub fn initial_info(&self) -> Result<CattleInitialConnect> {
        self.update();

        if let Ok(sys) = self.sys.read() {
            let disks = Disks::new_with_refreshed_list();
            let disk_bytes = disks.iter().map(|d| d.total_space()).sum();

            Ok(CattleInitialConnect {
                name: System::name().unwrap_or_default(),
                id: self.id,
                os_name: "".to_string(),
                os_version: System::os_version().unwrap_or_default(),
                os_version_long: System::long_os_version().unwrap_or_default(),
                ram_bytes: sys.total_memory(),
                disk_bytes,
                cpu_count: sys.cpus().len(),
                cpu_brand: sys.global_cpu_info().brand().to_string(),
                cpu_name: sys.global_cpu_info().name().to_string(),
                uptime: Duration::from_secs(System::uptime()),
            })
        } else {
            bail!("Failed to get a lock on system information handle")
        }
    }
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

    let state = CattleState::default();
    state.update();

    println!("{:?}", state.initial_info().unwrap());

    Ok(ExitCode::SUCCESS)
}
