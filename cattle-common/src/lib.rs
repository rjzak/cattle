use std::time::Duration;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Configuration file
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub mode: Mode,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum Mode {
    /// Push: send data to a remote host as some interval
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Pull {
    /// Port to listen on for remote connections
    pub listen: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct HerderPolls {
    /// Cattle to be polled, if Cattle are in Pull mode
    pub cattle: Vec<String>,

    /// Interval in seconds for polling for data
    pub interval_seconds: u32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Message(MessageType);

#[derive(Debug, Deserialize, Serialize)]
pub enum MessageType {
    RequestUpdate,
    SendUpdate,
    SendInitialInfo(CattleInitialConnect),
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CattleInitialConnect {
    /// Host name of the system
    pub name: String,

    /// Unchanging ID of the system
    pub id: Uuid,

    /// Operating System name
    pub os_name: String,

    /// Operating System version
    pub os_version: String,

    /// More detailed Operating System version
    pub os_version_long: String,

    /// Total RAM
    pub ram_bytes: u64,

    /// Total disk capacity
    pub disk_bytes: u64,

    /// Number of processors
    pub cpu_count: usize,

    /// CPU info
    pub cpu_brand: String,

    /// Name of the CPU
    pub cpu_name: String,

    /// OS-reported system uptime
    pub uptime: Duration,
}
