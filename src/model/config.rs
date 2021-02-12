use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
    pub website: String,
    pub leetcode: WebsiteConfig,
    pub live: LiveConfig,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct LiveConfig {
    pub interval: u64,
    pub last: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Settings {
    pub config: Config,
    pub verbose: bool,
}

impl Config {
    pub fn to_json(&self) -> String {
        return serde_json::to_string(self).unwrap();
    }

    pub fn write_to_file(&self, path: &str) -> bool {
        match fs::File::create(path) {
            Ok(file) => match serde_json::to_writer(file, self) {
                Ok(_) => {
                    return true;
                }
                Err(err) => {
                    format!(
                        "[ERROR] paring config file error, path={}, err={}",
                        path, err
                    );
                    return false;
                }
            },
            Err(err) => {
                format!(
                    "[ERROR] creating config file error, path={}, err={}",
                    path, err
                );
                return false;
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WebsiteConfig {
    pub users: Vec<String>,
    pub contests: Vec<String>,

    pub live_contests: Vec<String>,
    pub live_users: Vec<String>,

    pub max_rank: u32,
    pub concurrent: u32,
    pub cache: bool,
}
