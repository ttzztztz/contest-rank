use io::Write;
use serde::{de::DeserializeOwned, Serialize};
use std::io;
use std::{collections::HashMap, fs::File};

type CacheFileType = HashMap<String, String>;
const CACHE_FILE_PATH: &str = "./cache.json";

fn clear_cache_file() -> Result<File, io::Error> {
    match File::create(CACHE_FILE_PATH) {
        Ok(mut file) => {
            match file.write("{}".as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    println!("[WARN] Err when write initial data to file, e={}", e);
                }
            }
            return Ok(file);
        }
        Err(err) => {
            return Err(err);
        }
    }
}

fn clear_cache_file_suppress_error() {
    if let Err(e) = clear_cache_file() {
        println!(
            "[WARNING] Error when clear cache file, in suppress error function, e={}",
            e
        );
    }
}

fn open_cache_file() -> Result<File, io::Error> {
    match File::open(CACHE_FILE_PATH) {
        Ok(file) => {
            return Ok(file);
        }
        Err(e) => {
            println!("[WARN] Error when opening cache file e={}", e);
            clear_cache_file_suppress_error();
            return Err(e);
        }
    }
}

fn read_cache_file() -> CacheFileType {
    if let Ok(cache_file) = open_cache_file() {
        match serde_json::from_reader::<File, CacheFileType>(cache_file) {
            Ok(obj) => {
                return obj as CacheFileType;
            }
            Err(e) => {
                println!(
                    "[WARN] Error when parsing cache file, cache file truncated e={}",
                    e
                );
                return CacheFileType::new();
            }
        }
    }
    return CacheFileType::new();
}

fn write_cache_file(data: CacheFileType) {
    match File::create(CACHE_FILE_PATH) {
        Ok(file) => match serde_json::to_writer(file, &data) {
            Ok(_) => {}
            Err(e) => {
                println!("[WARN] Error when parsing & writing cache file e={}", e);
            }
        },
        Err(e) => {
            println!("[WARN] Error when writing cache file e={}", e);
        }
    }
}

pub fn clear_cache() -> Result<(), io::Error> {
    match clear_cache_file() {
        Ok(_) => {
            return Ok(());
        }
        Err(e) => {
            return Err(e);
        }
    }
}

pub fn set_cache<T>(key: &str, value: &T)
where
    T: DeserializeOwned + Serialize,
{
    match serde_json::to_string(&value) {
        Ok(value_json) => {
            let mut cache_file_data = read_cache_file();
            cache_file_data.insert(key.to_string(), value_json);
            write_cache_file(cache_file_data);
        }
        Err(e) => {
            println!("[WARN] Error when writing cache key={}, e={}", key, e);
        }
    }
}

pub fn get_cache<T>(key: &str) -> Option<T>
where
    T: DeserializeOwned + Serialize,
{
    let cache_file = read_cache_file();
    match cache_file.get(key) {
        None => {
            return None;
        }
        Some(buf) => match serde_json::from_str(buf.as_str()) {
            Err(e) => {
                println!("[WARN] Error when parsing cache key={}, e={}", key, e);
                return None;
            }
            Ok(val) => {
                return Some(val);
            }
        },
    }
}
