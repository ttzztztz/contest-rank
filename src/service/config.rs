use crate::model::config;
use std::fs::File;

pub fn read_config(path: &str) -> config::Config {
    let default_config = config::Config {
        leetcode: config::LeetcodeConfig {
            users: vec![],
            contests: vec![],

            live_users: vec![],
            live_contests: vec![],

            cache: false,
            max_rank: 2000,
        },
        website: String::from(""),
        live: config::LiveConfig {
            interval: 600u64,
            last: 5400i64,
        },
    };

    match File::open(path) {
        Ok(config_file) => match serde_json::from_reader::<File, config::Config>(config_file) {
            Ok(current_config) => return current_config,
            Err(err) => println!(
                "[ERROR] when parsing config, use default config instead, err = {}",
                err
            ),
        },
        Err(err) => println!(
            "[ERROR] when reading config, use default config instead, err = {}",
            err
        ),
    }

    return default_config;
}
