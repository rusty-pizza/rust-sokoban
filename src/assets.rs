//! Structs related to asset management.

#![allow(dead_code)]

use std::{
    fs::File,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use sfml::{
    audio::SoundBuffer,
    graphics::{Color, Font},
    SfBox,
};
use tiled::{Loader, Map};

use crate::graphics::Tilesheet;

pub const MOVE_SOUND_DIR: &str = "assets/sound/move";
pub const UNDO_SOUND_DIR: &str = "assets/sound/undo";
pub const UI_CLICK_SOUND_PATH: &str = "assets/sound/ui_click.ogg";
pub const WIN_FONT_PATH: &str = "assets/fonts/Varela_Round/VarelaRound-Regular.ttf";
pub const ICON_TILESHEET_PATH: &str = "assets/tilesheets/icons.tsx";
pub const MAIN_MENU_PATH: &str = "assets/levels/main_menu.tmx";
pub const PLAY_OVERLAY_PATH: &str = "assets/levels/overlay.tmx";

pub struct LevelCategory {
    pub name: String,
    pub color: Color,
    pub maps: Vec<(Map, PathBuf)>,
}

pub struct AssetManager {
    pub main_menu: Map,
    pub level_categories: Vec<LevelCategory>,
    pub icon_tilesheet: Tilesheet,
    pub walk_sounds: Vec<SfBox<SoundBuffer>>,
    pub undo_sounds: Vec<SfBox<SoundBuffer>>,
    pub ui_click_sound: SfBox<SoundBuffer>,
    pub tilesheet: Tilesheet,
    pub win_font: SfBox<Font>,
    pub play_overlay_map: Map,
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
                            let path = Path::new("assets/levels/").join(Path::new(path));
                            Ok((Loader::new().load_tmx_map(&path)?, path))
                        })
                        .collect::<Result<Vec<_>, tiled::Error>>()?,
                })
            }
        }

        let level_categories: Vec<RonLevelCategory> =
            ron::de::from_reader(File::open("assets/levels/levels.ron")?)?;

        let level_categories = level_categories
            .into_iter()
            .map(|lvl| lvl.try_into())
            .collect::<Result<Vec<LevelCategory>, _>>()?;

        let play_overlay_map = Loader::new().load_tmx_map(Path::new(PLAY_OVERLAY_PATH))?;

        let map = Loader::new().load_tmx_map(Path::new("assets/levels/test.tmx"))?;
        Ok(Self {
            tilesheet: Tilesheet::from_tileset(map.tilesets().first().unwrap().clone())?,
            main_menu: Loader::new().load_tmx_map(Path::new(MAIN_MENU_PATH))?,
            icon_tilesheet: Tilesheet::from_file(Path::new(ICON_TILESHEET_PATH))?,
            total_level_count: level_categories.iter().flat_map(|c| c.maps.iter()).count(),
            level_categories,
            play_overlay_map,
            ui_click_sound: SoundBuffer::from_file(UI_CLICK_SOUND_PATH)
                .expect("could not load ui click sfx"),
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

    /// Get a reference to the asset manager's total level count.
    pub fn total_level_count(&self) -> usize {
        self.total_level_count
    }
}
