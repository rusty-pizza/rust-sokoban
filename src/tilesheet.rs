use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use sfml::{
    graphics::{FloatRect, IntRect, Sprite, Texture},
    system::Vector2u,
    SfBox,
};
use tiled::{TiledError, Tileset};

use thiserror::Error;

pub struct Tilesheet {
    texture: SfBox<Texture>,
    tileset: Tileset,
}

#[derive(Debug, Error)]
pub enum TilesheetLoadError {
    #[error("IO error: {0}")]
    IoError(std::io::Error),
    #[error("Tiled error: {0}")]
    TiledError(TiledError),
    #[error("Invalid texture count")]
    InvalidTextureCount,
    #[error("Invalid texture path: {0:?}")]
    InvalidTexturePath(PathBuf),
}

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
    pub fn from_tileset<'p>(
        tileset: Tileset,
        origin_path: &'p Path,
    ) -> Result<Self, TilesheetLoadError> {
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

        Ok(Tilesheet {
            texture: texture,
            tileset,
        })
    }

    pub fn from_file<'p>(path: &'p Path) -> Result<Self, TilesheetLoadError> {
        let tileset = {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            tiled::parse_tileset(reader, 1)?
        };

        Self::from_tileset(tileset, path.parent().unwrap())
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn tile_rect(&self, gid: u32) -> Option<IntRect> {
        if gid == 0 {
            return None;
        }
        let id = gid - 1;

        let tile_width = self.tileset.tile_width;
        let tile_height = self.tileset.tile_height;
        let tiles_per_row = self.texture.size().x / tile_width;
        let x = id % tiles_per_row * tile_width;
        let y = id / tiles_per_row * tile_height;

        Some(IntRect {
            left: x as i32,
            top: y as i32,
            width: tile_width as i32,
            height: tile_height as i32,
        })
    }

    pub fn tile_uv(&self, gid: u32) -> Option<FloatRect> {
        if let Some(IntRect {
            left,
            top,
            width,
            height,
        }) = self.tile_rect(gid)
        {
            // In SFML, UVs are in pixel coordinates, so we just grab the tile rect and convert it
            // into a FloatRect
            Some(FloatRect {
                left: left as f32,
                top: top as f32,
                width: width as f32,
                height: height as f32,
            })
        } else {
            None
        }
    }

    pub fn tile_size(&self) -> Vector2u {
        Vector2u::new(self.tileset.tile_width, self.tileset.tile_height)
    }

    pub fn tile_sprite(&self, gid: u32) -> Option<Sprite> {
        if let Some(rect) = self.tile_rect(gid) {
            Some(Sprite::with_texture_and_rect(&self.texture, &rect))
        } else {
            None
        }
    }
}
