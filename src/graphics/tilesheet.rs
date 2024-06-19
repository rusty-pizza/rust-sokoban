use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use sfml::{
    graphics::{FloatRect, IntRect, Sprite, Texture},
    system::Vector2u,
    SfBox,
};

use thiserror::Error;
use tiled::{Error, Loader, Tileset};

/// A container for a tileset and the texture it references.
pub struct Tilesheet {
    texture: SfBox<Texture>,
    tileset: Arc<Tileset>,
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
        Error,
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
    pub fn from_tileset(tileset: Arc<Tileset>) -> Result<Self, TilesheetLoadError> {
        let tileset_image = tileset
            .image
            .as_ref()
            .ok_or(TilesheetLoadError::InvalidTextureCount)?;

        let mut texture = {
            let texture_path = Path::new(&tileset_image.source);

            Texture::from_file(texture_path.to_str().expect("obtaining valid UTF-8 path")).or(
                Err(TilesheetLoadError::InvalidTexturePath(
                    texture_path.to_owned(),
                )),
            )?
        };

        texture.set_smooth(true);
        texture.generate_mipmap();

        Ok(Tilesheet { texture, tileset })
    }

    /// Load a tilesheet from a path to a Tiled tileset, loading its texture along the way.
    pub fn from_file(path: &Path) -> Result<Self, TilesheetLoadError> {
        let tileset = Arc::new(Loader::new().load_tsx_tileset(path)?);

        Self::from_tileset(tileset)
    }

    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn tileset(&self) -> &Tileset {
        &self.tileset
    }

    pub fn tile_rect(&self, id: u32) -> Option<IntRect> {
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

    pub fn tile_uv(&self, id: u32) -> Option<FloatRect> {
        if let Some(IntRect {
            left,
            top,
            width,
            height,
        }) = self.tile_rect(id)
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

    pub fn tile_sprite(&self, id: u32) -> Option<Sprite> {
        self.tile_rect(id)
            .map(|rect| Sprite::with_texture_and_rect(&self.texture, rect))
    }
}
