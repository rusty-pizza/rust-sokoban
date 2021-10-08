use std::{
    error::Error,
    fmt::Display,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use sfml::{
    graphics::{FloatRect, Texture},
    system::Vector2u,
    SfBox,
};
use tiled::{TiledError, Tileset};

pub struct Tilesheet {
    texture: SfBox<Texture>,
    tileset: Tileset,
}

#[derive(Debug)]
pub enum TilesheetLoadError {
    IoError(std::io::Error),
    TiledError(TiledError),
    InvalidTextureCount,
    InvalidTexturePath(PathBuf),
}

impl Display for TilesheetLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TilesheetLoadError::IoError(err) => f.write_fmt(format_args!("IO error: {}", err)),
            TilesheetLoadError::TiledError(err) => {
                f.write_fmt(format_args!("Tiled error: {}", err))
            }
            TilesheetLoadError::InvalidTextureCount => f.write_str("Invalid texture count"),
            TilesheetLoadError::InvalidTexturePath(path) => {
                f.write_fmt(format_args!("Invalid texture path: {:?}", path))
            }
        }
    }
}

impl Error for TilesheetLoadError {}

impl From<std::io::Error> for TilesheetLoadError {
    fn from(x: std::io::Error) -> Self {
        Self::IoError(x)
    }
}

impl From<TiledError> for TilesheetLoadError {
    fn from(x: TiledError) -> Self {
        Self::TiledError(x)
    }
}

impl Tilesheet {
    pub fn from_tileset(
        tileset: Tileset,
        origin_path: &Path,
    ) -> Result<Tilesheet, TilesheetLoadError> {
        if tileset.images.len() != 1 {
            return Err(TilesheetLoadError::InvalidTextureCount);
        }

        let texture = {
            let texture_path = origin_path.join(Path::new(&tileset.images.first().unwrap().source));
            match Texture::from_file(texture_path.to_str().expect("obtaining valid UTF-8 path")) {
                Some(tex) => tex,
                _ => return Err(TilesheetLoadError::InvalidTexturePath(texture_path)),
            }
        };

        Ok(Tilesheet { texture, tileset })
    }

    pub fn from_file(path: &Path) -> Result<Tilesheet, TilesheetLoadError> {
        let tileset = {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            tiled::parse_tileset(reader, 1)?
        };

        Self::from_tileset(tileset, path)
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn tile_uv(&self, gid: u32) -> Option<FloatRect> {
        if gid == 0 {
            return None;
        }
        let id = gid - 1;

        let tile_width = self.tileset.tile_width;
        let tile_height = self.tileset.tile_height;
        let tiles_per_row = self.texture.size().x / tile_width;
        let x = (id % tiles_per_row * tile_width) as f32;
        let y = (id / tiles_per_row * tile_height) as f32;

        Some(FloatRect {
            left: x,
            top: y,
            width: tile_width as f32,
            height: tile_height as f32,
        })
    }

    pub fn tile_size(&self) -> Vector2u {
        Vector2u::new(self.tileset.tile_width, self.tileset.tile_height)
    }
}
