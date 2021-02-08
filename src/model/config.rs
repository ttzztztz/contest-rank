use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub website: String,
    pub leetcode: LeetcodeConfig,
}

pub trait WebsiteConfig {}

#[derive(Serialize, Deserialize, Clone)]
pub struct LiveConfig {
    pub interval: i32,
    pub contest: String,
    pub last: i32,
}

pub struct Settings {
    pub config: Config,
    pub verbose: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LeetcodeConfig {
    pub users: Vec<String>,
    pub contests: Vec<String>,
    pub live: LiveConfig,
    pub max_rank: u32,
    pub cache: bool,
}

impl WebsiteConfig for LeetcodeConfig {}
