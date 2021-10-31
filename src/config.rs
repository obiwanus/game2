use std::fs;

use miniserde::{json, Deserialize, Serialize};

use crate::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub heightmap_path: String,
    pub start_with_flat_terrain: bool,
}

impl Config {
    pub fn load_or_default() -> Result<Self> {
        let config = if let Ok(config) = fs::read_to_string("config.json") {
            json::from_str(&config)?
        } else {
            Config {
                heightmap_path: "textures/heightmaps/heightmap.png".to_owned(),
                start_with_flat_terrain: true,
            }
        };
        Ok(config)
    }

    pub fn save(&self) {
        let string = json::to_string(self);
        fs::write("config.json", string).unwrap();
    }
}
