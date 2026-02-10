use std::ops::Add;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::{bail, Context, Result};
use clap::Parser;
use p384::{PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use sysinfo::{Disks, Pid, System};
use uuid::Uuid;

use cattle_common::{CattleInitialConnect, CattleUpdate, Mode};

#[cfg(target_os = "macos")]
pub const ID_FILE: &str = "/Library/Preferences/Cattle/id.bin";
#[cfg(all(target_family = "unix", not(target_os = "macos")))]
pub const ID_FILE: &str = "/var/cache/cattle-herder/id.bin";
#[cfg(target_os = "windows")]
pub const ID_FILE: &str = "C:\\cattle-herder\\id.bin";

pub const VERSION: &str = concat!(env!("CATTLE_VERSION"), " ", env!("CATTLE_BUILD_DATE"));

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

        let mut system = System::new_all();
        system.refresh_all();
        system.refresh_cpu_all();

        // Wait a bit because CPU usage is based on diff.
        std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
        system.refresh_cpu_all();

        CattleState {
            sys: Arc::new(RwLock::new(system)),
            id: uuid,
            pkey,
            skey,
        }
    }
}

impl CattleState {
    fn system_update(&self) -> Result<()> {
        if let Ok(mut sys) = self.sys.write() {
            sys.refresh_all();
            sys.refresh_cpu_all();

            // Wait a bit because CPU usage is based on diff.
            std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
            sys.refresh_cpu_all();
        } else {
            bail!("Failed to get read-write lock on System");
        }

        Ok(())
    }

    pub fn initial_info(&self) -> Result<CattleInitialConnect> {
        self.system_update()?;

        if let Ok(sys) = self.sys.read() {
            let disks = Disks::new_with_refreshed_list();
            let disk_bytes = disks.iter().map(|d| d.total_space()).sum();

            Ok(CattleInitialConnect {
                os_name: System::name().unwrap_or_default(),
                hostname: System::host_name().unwrap(),
                id: self.id,
                os_version: System::os_version().unwrap_or_default(),
                os_version_long: System::long_os_version().unwrap_or_default(),
                ram_bytes: sys.total_memory(),
                disk_bytes,
                cpu_count: sys.cpus().len() as u64,
                cpu_brand: sys.cpus()[0].brand().to_string(),
                cpu_name: sys.cpus()[0].name().to_string(),
                uptime: Duration::from_secs(System::uptime()),
            })
        } else {
            bail!("Failed to get a lock on system information handle")
        }
    }

    pub fn periodic_update(&self) -> Result<CattleUpdate> {
        self.system_update()?;

        if let Ok(sys) = self.sys.read() {
            let disks = Disks::new_with_refreshed_list();
            let available_bytes = disks.iter().map(|d| d.available_space()).sum();

            let mut biggest_process_pid = Pid::from_u32(0);
            let mut biggest_process_usage = 0f32;
            for (_, process) in sys.processes().iter() {
                if process.cpu_usage() > biggest_process_usage {
                    biggest_process_pid = process.pid();
                    biggest_process_usage = process.cpu_usage();
                }
            }

            let biggest_process_owner =
                sys.process(biggest_process_pid).unwrap().user_id().unwrap();
            let biggest_process_name = sys.process(biggest_process_pid).unwrap().name();
            let biggest_process_owner = users::get_user_by_uid(biggest_process_owner.add(0))
                .unwrap()
                .name()
                .to_str()
                .unwrap()
                .to_string();

            Ok(CattleUpdate {
                cpu_utilization: sys.global_cpu_usage(),
                available_memory_bytes: sys.available_memory(),
                available_disk_bytes: available_bytes,
                running_processes: sys.processes().len() as u64,
                most_intense_process_name: biggest_process_name.to_str().unwrap().to_string(),
                most_intense_process_owner: biggest_process_owner,
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
    println!("{:?}", state.initial_info().unwrap());

    Ok(ExitCode::SUCCESS)
}

#[cfg(test)]
mod tests {
    use crate::CattleState;

    #[test]
    #[ignore]
    fn state() {
        let state = CattleState::default();
        state
            .system_update()
            .expect("failed to update system information");
        println!("{:?}", state.initial_info().unwrap());

        let update = state.periodic_update().unwrap();
        println!("{:?}", update);
    }
}
