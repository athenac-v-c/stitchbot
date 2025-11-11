use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub rpc_url: String,
    pub p2p_port: u16,
    pub p2p_bootstrap_peers: Vec<String>,
    pub stitch_reward_sompi: u64,
    pub dag_window: usize,
    pub min_blue_delta: u64,
    pub rate_limit_seconds: u32,
}

impl Config {
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }
}
