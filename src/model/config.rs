use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub website: String,
    pub leetcode: LeetcodeConfig,
    pub live: LiveConfig,
}

pub trait WebsiteConfig {}

#[derive(Serialize, Deserialize, Clone)]
pub struct LiveConfig {
    pub interval: u64,
    pub last: i64,
}

pub struct Settings {
    pub config: Config,
    pub verbose: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LeetcodeConfig {
    pub users: Vec<String>,
    pub contests: Vec<String>,

    pub live_contests: Vec<String>,
    pub live_users: Vec<String>,
    pub max_rank: u32,
    pub concurrent: u32,
    pub cache: bool,
}

impl WebsiteConfig for LeetcodeConfig {}
