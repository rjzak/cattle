use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEFAULT_PORT: u16 = 6543;

/// Configuration file
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(flatten)]
    pub mode: Mode,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Mode {
    /// Push: Cattle sends data to a monitor at some interval
    Push(CattlePush),

    /// Pull: receive data from clients (Herder or Cattle)
    Pull(Pull),

    /// Poll: herder quests data at some interval
    Poll(HerderPolls),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CattlePush {
    /// Server IP or hostname
    pub server: String,

    /// Server port
    pub port: u16,

    /// Interval in seconds for sending data
    pub interval_seconds: u32,
}

impl Default for CattlePush {
    fn default() -> Self {
        CattlePush {
            server: "localhost".to_string(),
            port: DEFAULT_PORT,
            interval_seconds: 10,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pull {
    /// Port to listen on for remote connections
    pub listen: u16,
}

impl Default for Pull {
    fn default() -> Self {
        Pull {
            listen: DEFAULT_PORT,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HerderPolls {
    /// Cattle to be polled, if Cattle are in Pull mode
    pub cattle: Vec<String>,

    /// Interval in seconds for polling for data
    pub interval_seconds: u32,
}

impl Default for HerderPolls {
    fn default() -> Self {
        HerderPolls {
            cattle: vec!["localhost".to_string()],
            interval_seconds: 10,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message(MessageType);

#[derive(Debug, Deserialize, Serialize)]
pub enum MessageType {
    SendPublicKey(Vec<u8>),
    RequestUpdate,
    SendUpdate(CattleUpdate),
    SendInitialInfo(CattleInitialConnect),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CattleInitialConnect {
    /// Host name of the system
    pub name: String,

    /// Unchanging ID of the system
    pub id: Uuid,

    /// Operating System version
    pub os_version: String,

    /// More detailed Operating System version
    pub os_version_long: String,

    /// Total RAM
    pub ram_bytes: u64,

    /// Total disk capacity
    pub disk_bytes: u64,

    /// Number of processors
    pub cpu_count: u64,

    /// CPU info
    pub cpu_brand: String,

    /// Name of the CPU
    pub cpu_name: String,

    /// OS-reported system uptime
    pub uptime: Duration,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CattleUpdate {
    /// CPU utilization
    pub cpu_utilization: f32,

    /// Unused RAM
    pub available_memory_bytes: u64,

    /// Unused disk space
    pub available_disk_bytes: u64,

    /// Number of processes running
    pub running_processes: u64,

    /// Process using most of the CPU
    pub most_intense_process_name: String,

    /// Username of the owner of the process using the CPU the most
    pub most_intense_process_owner: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::poll(
        "../example_configs/herder_polls.toml.example",
        Mode::Poll(HerderPolls::default())
    )]
    #[case::listen("../example_configs/listen.toml.example", Mode::Pull(Pull::default()))]
    #[case::listen(
        "../example_configs/push.toml.example",
        Mode::Push(CattlePush::default())
    )]
    #[test]
    fn config(#[case] test: &str, #[case] mode: Mode) {
        let config_string =
            std::fs::read_to_string(test).unwrap_or_else(|_| panic!("failed to read {test}"));

        match toml::from_str::<Config>(&config_string) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Error parsing {test}: {e}, regenerating default");
                let d = Config { mode };
                let d = toml::to_string(&d).unwrap();
                std::fs::write(test, d).unwrap();
            }
        }
    }
}
