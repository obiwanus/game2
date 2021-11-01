use std::fs;

use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub heightmap_path: String,
    pub start_with_flat_terrain: bool,
}

impl Config {
    pub fn load_or_default() -> Result<Self> {
        let config = if let Ok(config) = fs::read_to_string("config.json") {
            serde_json::from_str(&config)?
        } else {
            Config {
                heightmap_path: "textures/heightmaps/heightmap.png".to_owned(),
                start_with_flat_terrain: true,
            }
        };
        Ok(config)
    }

    pub fn save(&self) {
        let string = serde_json::to_string(self).unwrap();

        fs::write("config.json", string).unwrap();
    }
}
