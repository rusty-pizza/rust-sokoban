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

/// A container for a tileset and the texture it references.
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
    #[error("Invalid texture count: Tileset must have a minimum of one image, and multiple images are currently not supported")]
    InvalidTextureCount,
    #[error("Invalid texture path: {0:?}")]
    InvalidTexturePath(PathBuf),
    #[error("The tileset provided has an invalid source path: {0:?}")]
    TilesetHasInvalidSource(Option<PathBuf>),
}

impl Tilesheet {
    /// Create a tilesheet from a Tiled tileset, loading its texture along the way.
    pub fn from_tileset(tileset: Tileset) -> Result<Self, TilesheetLoadError> {
        let tileset_image = tileset
            .images
            .first()
            .ok_or(TilesheetLoadError::InvalidTextureCount)?;

        let mut texture = {
            let origin_path = match &tileset.source {
                Some(path) => path.parent().ok_or_else(|| {
                    TilesheetLoadError::TilesetHasInvalidSource(Some(path.clone()))
                })?,
                None => return Err(TilesheetLoadError::TilesetHasInvalidSource(None)),
            };

            let texture_path = origin_path.join(Path::new(&tileset_image.source));

            Texture::from_file(texture_path.to_str().expect("obtaining valid UTF-8 path"))
                .or(Err(TilesheetLoadError::InvalidTexturePath(texture_path)))?
        };

        texture.set_smooth(true);
        texture.generate_mipmap();

        Ok(Tilesheet { texture, tileset })
    }

    /// Load a tilesheet from a path to a Tiled tileset, loading its texture along the way.
    pub fn from_file(path: &Path, first_gid: Gid) -> Result<Self, TilesheetLoadError> {
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

        let spacing = self.tileset.spacing;
        let tile_width = self.tileset.tile_width;
        let tile_height = self.tileset.tile_height;
        let tiles_per_row = (self.texture.size().x + spacing) / (tile_width + spacing);
        let x = (id % tiles_per_row) * (tile_width + spacing);
        let y = (id / tiles_per_row) * (tile_height + spacing);

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
        self.tile_rect(gid)
            .map(|rect| Sprite::with_texture_and_rect(&self.texture, &rect))
    }
}
