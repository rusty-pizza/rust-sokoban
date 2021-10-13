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
use tiled::{error::TiledError, tile::Gid, tileset::Tileset};

use thiserror::Error;

pub struct Tilesheet {
    texture: SfBox<Texture>,
    tileset: Tileset,
}

#[derive(Debug, Error)]
pub enum TilesheetLoadError {
    #[error("IO error: {0}")]
    IoError(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("Tiled error: {0}")]
    TiledError(
        #[from]
        #[source]
        TiledError,
    ),
    #[error("Invalid texture count")]
    InvalidTextureCount,
    #[error("Invalid texture path: {0:?}")]
    InvalidTexturePath(PathBuf),
    #[error("The tileset provided has an invalid source path: {0:?}")]
    TilesetHasInvalidSource(Option<PathBuf>),
}

impl Tilesheet {
    pub fn from_tileset<'p>(tileset: Tileset) -> Result<Self, TilesheetLoadError> {
        if tileset.images.len() != 1 {
            return Err(TilesheetLoadError::InvalidTextureCount);
        }

        let texture = {
            let origin_path = match &tileset.source {
                Some(path) => match path.parent() {
                    Some(parent) => parent.to_owned(),
                    None => {
                        return Err(TilesheetLoadError::TilesetHasInvalidSource(Some(
                            path.clone(),
                        )))
                    }
                },
                None => return Err(TilesheetLoadError::TilesetHasInvalidSource(None)),
            };
            let texture_path = origin_path.join(Path::new(&tileset.images.first().unwrap().source));
            match Texture::from_file(texture_path.to_str().expect("obtaining valid UTF-8 path")) {
                Some(tex) => tex,
                _ => return Err(TilesheetLoadError::InvalidTexturePath(texture_path)),
            }
        };

        Ok(Tilesheet { texture, tileset })
    }

    pub fn from_file<'p>(path: &'p Path, first_gid: Gid) -> Result<Self, TilesheetLoadError> {
        let tileset = {
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            Tileset::parse_reader(reader, first_gid, Some(path))?
        };

        Self::from_tileset(tileset)
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn tileset(&self) -> &Tileset {
        &self.tileset
    }

    pub fn tile_rect(&self, gid: Gid) -> Option<IntRect> {
        if gid == Gid::EMPTY {
            return None;
        }
        let id = gid.0 - self.tileset.first_gid.0;

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

    pub fn tile_uv(&self, gid: Gid) -> Option<FloatRect> {
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

    pub fn tile_sprite(&self, gid: Gid) -> Option<Sprite> {
        if let Some(rect) = self.tile_rect(gid) {
            Some(Sprite::with_texture_and_rect(&self.texture, &rect))
        } else {
            None
        }
    }
}
