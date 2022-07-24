//! Structs related to asset management.

#![allow(dead_code)]

use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use serde::Deserialize;
use sfml::{
    audio::SoundBuffer,
    graphics::{Color, Font},
    SfBox,
};
use tiled::{map::Map, tile::Gid};

use crate::graphics::Tilesheet;

pub const MOVE_SOUND_DIR: &str = "assets/sound/move";
pub const UNDO_SOUND_DIR: &str = "assets/sound/undo";
pub const UI_CLICK_SOUND_PATH: &str = "assets/sound/ui_click.ogg";
pub const WIN_FONT_PATH: &str = "assets/fonts/Varela_Round/VarelaRound-Regular.ttf";
pub const ICON_TILESHEET_PATH: &str = "assets/tilesheets/icons.tsx";
pub const MAIN_MENU_PATH: &str = "assets/levels/main_menu.tmx";

pub struct LevelCategory {
    pub name: String,
    pub color: Color,
    pub maps: Vec<Map>,
}

pub struct AssetManager {
    pub main_menu: Map,
    pub level_categories: Vec<LevelCategory>,
    pub icon_tilesheet: Tilesheet,
    pub walk_sounds: Vec<Vec<u8>>,
    pub undo_sounds: Vec<Vec<u8>>,
    pub ui_click_sound: Vec<u8>,
    pub tilesheet: Tilesheet,
    pub win_font: SfBox<Font>,
    total_level_count: usize,
}

impl AssetManager {
    /// Creates a new asset manager and loads the data it references.
    pub fn load() -> anyhow::Result<Self> {
        #[derive(Deserialize)]
        pub struct RonLevelCategory {
            pub name: String,
            pub color: u32,
            pub maps: Vec<String>,
        }

        impl TryFrom<RonLevelCategory> for LevelCategory {
            type Error = anyhow::Error;

            fn try_from(value: RonLevelCategory) -> Result<Self, Self::Error> {
                Ok(LevelCategory {
                    name: value.name,
                    color: Color::from(value.color),
                    maps: value
                        .maps
                        .iter()
                        .map(|path| {
                            Map::parse_file(&Path::new("assets/levels/").join(&Path::new(path)))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                })
            }
        }

        let level_categories: Vec<RonLevelCategory> =
            ron::de::from_reader(File::open("assets/levels/levels.ron")?)?;

        let level_categories = level_categories
            .into_iter()
            .map(|lvl| lvl.try_into())
            .collect::<Result<Vec<LevelCategory>, _>>()?;

        let map = Map::parse_file(Path::new("assets/levels/test.tmx"))?;

        let mut ui_click_sound = Vec::new();
        File::open(UI_CLICK_SOUND_PATH)?.read_to_end(&mut ui_click_sound);

        let mut walk_sounds = Vec::new();
        for entry in std::fs::read_dir(Path::new(MOVE_SOUND_DIR))
            .expect("could not inspect the sounds directory")
        {
            let mut sound = Vec::new();
            File::open(entry?.path())?.read_to_end(&mut sound);
            walk_sounds.push(sound);
        }

        let mut undo_sounds = Vec::new();
        for entry in std::fs::read_dir(Path::new(UNDO_SOUND_DIR))
            .expect("could not inspect the sounds directory")
        {
            let mut sound = Vec::new();
            File::open(entry?.path())?.read_to_end(&mut sound);
            undo_sounds.push(sound);
        }

        Ok(Self {
            tilesheet: Tilesheet::from_tileset(map.tilesets[0].clone())?,
            main_menu: Map::parse_file(Path::new(MAIN_MENU_PATH))?,
            icon_tilesheet: Tilesheet::from_file(Path::new(ICON_TILESHEET_PATH), Gid(1))?,
            total_level_count: level_categories.iter().flat_map(|c| c.maps.iter()).count(),
            level_categories,
            ui_click_sound,
            walk_sounds,
            undo_sounds,
            win_font: Font::from_file(WIN_FONT_PATH).expect("could not load win font"),
        })
    }

    /// Get a reference to the asset manager's total level count.
    pub fn total_level_count(&self) -> usize {
        self.total_level_count
    }
}
