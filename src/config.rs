use std::io::Read;

use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub(crate) scale_quality: bool,
    pub(crate) text_speed: f64,
    pub(crate) debug: bool,
    pub(crate) data_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            scale_quality: true,
            text_speed: 50.0,
            debug: false,
            data_path: String::from("data/"),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let file_path = "config.json";
        let mut file = std::fs::File::open(file_path).expect("Failed to open config file");
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        if let Ok(cfg) = serde_json::from_str(&contents) {
            Ok(cfg)
        } else {
            Err("Failed to parse config file".into())
        }
    }
}
