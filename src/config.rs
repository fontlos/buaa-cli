use serde::{Deserialize, Serialize};

use std::fs::OpenOptions;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Config {
    pub username: String,
    pub password: String,
    pub boya_token: String,
    pub class_token: String,
}

impl Config {
    pub fn new() -> Self {
        let path = crate::util::get_path("buaa-config.json").unwrap();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .unwrap();
        if let Ok(config) = serde_json::from_reader(file) {
            config
        } else {
            Self::default()
        }
    }
    pub fn is_valid(&self) -> bool {
        !self.username.is_empty() && !self.password.is_empty()
    }
    pub fn save(&self) {
        let path = crate::util::get_path("buaa-config.json").unwrap();
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)
            .unwrap();
        serde_json::to_writer(file, self).unwrap();
    }
}

impl Drop for Config {
    fn drop(&mut self) {
        self.save();
    }
}
