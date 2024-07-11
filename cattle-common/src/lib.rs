use serde::{Deserialize, Serialize};

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
