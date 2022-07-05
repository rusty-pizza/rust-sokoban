//! Structs related to asset management.

#![allow(dead_code)]

use std::path::Path;

use sfml::{audio::SoundBuffer, graphics::Font, SfBox};
use tiled::map::Map;

use crate::graphics::Tilesheet;

pub const LEVEL_PATHS: [&'static str; 12] = [
    "assets/levels/tutorial/movement.tmx",
    "assets/levels/tutorial/bricks.tmx",
    "assets/levels/tutorial/manouvering.tmx",
    "assets/levels/tutorial/manouvering2.tmx",
    "assets/levels/basic/room.tmx",
    "assets/levels/basic/plus.tmx",
    "assets/levels/holes/tutorial.tmx",
    "assets/levels/holes/tutorial2.tmx",
    "assets/levels/holes/pit.tmx",
    "assets/levels/holes/cliff.tmx",
    "assets/levels/holes/roundabout.tmx",
    "assets/levels/holes/alt_route.tmx",
];
pub const MOVE_SOUND_DIR: &'static str = "assets/sound/move";
pub const UNDO_SOUND_DIR: &'static str = "assets/sound/undo";
pub const WIN_FONT_PATH: &'static str = "assets/fonts/Varela_Round/VarelaRound-Regular.ttf";

pub struct AssetManager {
    pub maps: Vec<Map>,
    pub walk_sounds: Vec<SfBox<SoundBuffer>>,
    pub undo_sounds: Vec<SfBox<SoundBuffer>>,
    pub tilesheet: Tilesheet,
    pub win_font: SfBox<Font>,
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
            walk_sounds: std::fs::read_dir(Path::new(MOVE_SOUND_DIR))
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
            undo_sounds: std::fs::read_dir(Path::new(UNDO_SOUND_DIR))
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
            win_font: Font::from_file(WIN_FONT_PATH).expect("could not load win font"),
        })
    }
}
