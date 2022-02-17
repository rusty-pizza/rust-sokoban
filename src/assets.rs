//! Structs related to asset management.

#![allow(dead_code)]

use std::{
    collections::HashMap,
    convert::{TryFrom, TryInto},
    fmt::Debug,
    path::{Path, PathBuf},
};

use tiled::{error::TiledError, map::Map};

use crate::graphics::Tilesheet;

pub const LEVEL_PATHS: [&'static str; 2] = ["assets/levels/test.tmx", "assets/levels/untitled.tmx"];

pub struct AssetManager {
    pub maps: Vec<Map>,
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
        })
    }
}
