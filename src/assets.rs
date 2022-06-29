//! Structs related to asset management.

#![allow(dead_code)]

use std::path::Path;

use sfml::{audio::SoundBuffer, SfBox};
use tiled::map::Map;

use crate::graphics::Tilesheet;

pub const LEVEL_PATHS: [&'static str; 5] = [
    "assets/levels/tutorial/movement.tmx",
    "assets/levels/tutorial/bricks.tmx",
    "assets/levels/tutorial/manouvering.tmx",
    "assets/levels/basic/plus.tmx",
    "assets/levels/untitled.tmx",
];
pub const SOUND_DIR: &'static str = "assets/sound";

pub struct AssetManager {
    pub maps: Vec<Map>,
    pub walk_sounds: Vec<SfBox<SoundBuffer>>,
    pub tilesheet: Tilesheet,
}

impl AssetManager {
    /// Creates a new asset manager and loads the data it references.
    pub fn load() -> anyhow::Result<Self> {
        let map = Map::parse_file(Path::new("assets/levels/test.tmx"))?;
        Ok(Self {
            tilesheet: Tilesheet::from_tileset(map.tilesets[0].clone())?,
            maps: LEVEL_PATHS
                .iter()
                .map(|path| Map::parse_file(Path::new(path)))
                .collect::<Result<Vec<_>, _>>()?,
            walk_sounds: std::fs::read_dir(Path::new(SOUND_DIR))
                .expect("could not inspect the sounds directory")
                .map(|entry| {
                    entry
                        .expect("could not read file in sounds directory")
                        .path()
                })
                .map(|path| {
                    SoundBuffer::from_file(path.to_str().unwrap())
                        .expect("could not read sound file")
                })
                .collect(),
        })
    }
}
