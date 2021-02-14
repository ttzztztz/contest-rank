use crate::model::config;
use std::{fs::File, path::Path};

fn get_default_config() -> config::Config {
    config::Config {
        leetcode: config::WebsiteConfig {
            users: vec![],
            contests: vec![],

            live_users: vec![],
            live_contests: vec![],

            concurrent: 1,
            cache: false,
            max_rank: 2000,
        },
        website: String::from(""),
        live: config::LiveConfig {
            interval: 600u64,
            last: 5400i64,
        },
    }
}

pub fn read_config(path: &str) -> config::Config {
    let default_config = get_default_config();

    let config_file = Path::new(path);
    if !config_file.exists() {
        println!(
            "[INFO] Config file doesn't exist, will write to file path={}",
            path
        );
        
        default_config.write_to_file(path);
        return default_config;
    }

    match File::open(path) {
        Ok(config_file) => match serde_json::from_reader::<File, config::Config>(config_file) {
            Ok(current_config) => return current_config,
            Err(err) => println!(
                "[ERROR] When parsing config, use default config instead, err={}",
                err
            ),
        },
        Err(err) => println!(
            "[ERROR] When reading config, use default config instead, err={}",
            err
        ),
    }

    return default_config;
}
