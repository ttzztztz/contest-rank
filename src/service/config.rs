use super::super::model::config;
use std::fs::File;

pub fn read_config(path: String) -> config::Config {
    let default_config = config::Config {
        users: vec![],
        contests: vec![],
        website: String::from(""),
    };

    match File::open(path) {
        Ok(config_file) => match serde_json::from_reader::<File, config::Config>(config_file) {
            Ok(current_config) => return current_config,
            Err(err) => println!("[ERROR] when parsing config, use default config instead, err = {}", err),
        },
        Err(err) => println!("[ERROR] when reading config, use default config instead, err = {}", err),
    }

    return default_config;
}
