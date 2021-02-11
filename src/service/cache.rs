use serde::{de::DeserializeOwned, Serialize};
use std::{io, path::Path};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};

const CACHE_FILE_PATH: &str = "./cache";

macro_rules! cache_path {
    ($name: expr) => {
        format!("{}/{}.cache", CACHE_FILE_PATH, $name);
    };
}

pub async fn clear_cache_file() -> Result<(), io::Error> {
    return fs::remove_dir_all(CACHE_FILE_PATH).await;
}

async fn read_cache_file(key: &str) -> Result<fs::File, io::Error> {
    let path = cache_path!(key);
    return fs::File::open(path).await;
}

async fn write_cache_file(key: &str, data: &str) -> bool {
    let path = cache_path!(key);
    let cache_path = Path::new(CACHE_FILE_PATH);
    if !cache_path.exists() {
        println!("[INFO] cache path doesn't exist, mkdir={}", CACHE_FILE_PATH);
        if let Err(err) = fs::create_dir_all("./cache").await {
            println!(
                "[WARN] cache path doesn't exist, cannot make, path={}, e={}",
                CACHE_FILE_PATH, err
            );
            return false;
        }
    }

    if cache_path.is_file() {
        println!(
            "[WARN] cache path is not dir, is a file, path={}",
            CACHE_FILE_PATH
        );
        return false;
    }

    match fs::File::create(&path).await {
        Ok(mut file) => match file.write_all(data.as_bytes()).await {
            Ok(_) => {
                return true;
            }
            Err(err) => {
                println!("[ERROR] when writing cache file, path={}, e={}", path, err);
            }
        },
        Err(err) => {
            println!(
                "[ERROR] error when creating cache file, path={}, e={}",
                path, err
            );
        }
    }
    return false;
}

pub async fn set_cache<T>(key: &str, value: &T)
where
    T: DeserializeOwned + Serialize,
{
    match serde_json::to_string(&value) {
        Ok(value_json) => {
            write_cache_file(key, &value_json).await;
        }
        Err(err) => {
            println!("[WARN] Error when writing cache key={}, e={}", key, err);
        }
    }
}

pub async fn get_cache<T>(key: &str) -> Option<T>
where
    T: DeserializeOwned + Serialize,
{
    let path = cache_path!(key);
    match read_cache_file(key).await {
        Ok(mut cache_file) => {
            let mut buf: String = String::from("");
            if let Err(err) = cache_file.read_to_string(&mut buf).await {
                println!(
                    "[ERROR] Error when async read cache from key={}, e={}",
                    key, err
                );
                return None;
            }

            match serde_json::from_str(&buf) {
                Ok(val) => {
                    return Some(val);
                }
                Err(err) => {
                    println!("[WARN] Error when parsing cache key={}, e={}", key, err);
                    if let Err(err) = fs::remove_file(path).await {
                        println!(
                            "[WARN] Error when deleting cache file key={}, e={}",
                            key, err
                        );
                    }
                    return None;
                }
            }
        }
        Err(_) => {
            return None;
        }
    }
}

pub async fn force_clear_cache() {
    match clear_cache_file().await {
        Ok(_) => {
            println!("[INFO] 🌟 Cache cleared!");
        }
        Err(err) => {
            panic!("[ERROR] Error when clear cache dir e={}", err)
        }
    }
}
