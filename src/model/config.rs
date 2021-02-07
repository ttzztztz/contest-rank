use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub users: Vec<String>,
    pub contests: Vec<String>,
    pub website: String,
}
