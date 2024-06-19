//! Dynamic objects that are owned by the level, these being [`Crate`]s and [`Goal`]s.

#![allow(dead_code)]

use std::{fmt::Display, num::NonZeroU32};

use sfml::{
    graphics::{Drawable, Transformable},
    system::{Vector2f, Vector2i},
};
use tiled::PropertyValue;

use crate::{graphics::SpriteAtlas, graphics::Tilesheet};

pub(super) mod parsing;

/// When applied to a crate, the crate's type. When applied to a goal, the crate type
/// the goal accepts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CrateStyle(NonZeroU32);

#[derive(Debug)]
pub struct CrateStyleParseError;

impl Display for CrateStyleParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(
            "could not parse crate style from property as it is not a non-zero integer value",
        )
    }
}

impl std::error::Error for CrateStyleParseError {}

impl CrateStyle {
    pub(super) fn from_tiled_property(prop: &PropertyValue) -> Result<Self, CrateStyleParseError> {
        if let PropertyValue::IntValue(style) = prop {
            NonZeroU32::new(*style as u32)
                .map(CrateStyle)
                .ok_or(CrateStyleParseError)
        } else {
            Err(CrateStyleParseError)
        }
    }
}

#[derive(Clone, Copy)]
pub enum AcceptedCrateStyle {
    Specific(CrateStyle),
    Any,
}

impl AcceptedCrateStyle {
    pub fn accepts(self, style: CrateStyle) -> bool {
        match self {
            AcceptedCrateStyle::Specific(accepted) if accepted == style => true,
            AcceptedCrateStyle::Any => true,
            _ => false,
        }
    }
}

impl Default for AcceptedCrateStyle {
    fn default() -> Self {
        Self::Any
    }
}

/// A crate the player can move around.
#[derive(Clone)]
pub struct Crate<'s> {
    position: Vector2i,
    sprite_atlas: SpriteAtlas<'s>,
    style: CrateStyle,
    in_hole: bool,
    grid_size: Vector2f,
}

impl<'s> Crate<'s> {
    const NORMAL_FRAME: usize = 0;
    const DROPPED_FRAME: usize = 1;
    const POSITIONED_FRAME: usize = 2;
    const TRANSLUCENT_ALPHA: u8 = 150;

    pub fn new(
        position: Vector2i,
        tilesheet: &'s Tilesheet,
        id: u32,
        grid_size: Vector2f,
    ) -> Option<Self> {
        let tile = tilesheet.tileset().get_tile(id)?;

        let crate_type = match tile.properties.get("style") {
            Some(x) => CrateStyle::from_tiled_property(x).unwrap(),
            None => None?,
        };

        let get_frame_id = |frame: usize| -> Option<u32> {
            let frames = &tile.animation.as_ref()?;
            Some(frames.get(frame)?.tile_id)
        };

        let normal_tex_rect = tilesheet.tile_rect(id)?;
        let dropped_tex_rect = tilesheet.tile_rect(get_frame_id(Self::DROPPED_FRAME)?)?;
        let positioned_tex_rect = tilesheet.tile_rect(get_frame_id(Self::POSITIONED_FRAME)?)?;

        let sprite_atlas = {
            let mut sprite_atlas = SpriteAtlas::with_texture_and_frames(
                tilesheet.texture(),
                &[normal_tex_rect, dropped_tex_rect, positioned_tex_rect],
            );
            sprite_atlas.set_position(
                Vector2f::new(position.x as f32, position.y as f32).cwise_mul(grid_size),
            );
            sprite_atlas
        };

        Some(Self {
            position,
            style: crate_type,
            sprite_atlas,
            in_hole: false,
            grid_size,
        })
    }

    pub fn position(&self) -> Vector2i {
        self.position
    }

    pub fn set_position(&mut self, position: Vector2i) {
        self.position = position;
        self.sprite_atlas.set_position(
            Vector2f::new(position.x as f32, position.y as f32).cwise_mul(self.grid_size),
        );
    }

    pub fn in_hole(&self) -> bool {
        self.in_hole
    }

    pub fn set_in_hole(&mut self, in_hole: bool) {
        self.in_hole = in_hole;
        self.sprite_atlas
            .set_frame(
                in_hole
                    .then(|| Self::DROPPED_FRAME)
                    .unwrap_or(Self::NORMAL_FRAME),
            )
            .unwrap();
    }

    pub fn set_opaque(&mut self, val: bool) {
        self.sprite_atlas.set_alpha(if val {
            u8::MAX
        } else {
            Self::TRANSLUCENT_ALPHA
        });
    }

    pub fn set_is_positioned(&mut self, val: bool) {
        self.sprite_atlas
            .set_frame(if val {
                Self::POSITIONED_FRAME
            } else {
                Self::NORMAL_FRAME
            })
            .unwrap();
    }

    /// Get the crate's style.
    pub fn style(&self) -> CrateStyle {
        self.style
    }
}

impl<'s> Drawable for Crate<'s> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        self.sprite_atlas.draw(target, states);
    }
}

/// Indicates where a certain style of crate should be put in a level.
#[derive(Clone)]
pub struct Goal<'s> {
    position: Vector2i,
    accepted_style: AcceptedCrateStyle,
    sprite_atlas: SpriteAtlas<'s>,
}

impl<'s> Goal<'s> {
    const PENDING_FRAME: usize = 0;
    const DONE_FRAME: usize = 1;

    pub fn new(
        position: Vector2i,
        tilesheet: &'s Tilesheet,
        id: u32,
        grid_size: Vector2f,
    ) -> anyhow::Result<Self> {
        let tile = tilesheet
            .tileset()
            .get_tile(id)
            .ok_or_else(|| anyhow::anyhow!("goal tile gid does not exist in tilesheet"))?;

        let get_frame_id = |frame: usize| -> Option<u32> {
            let frames = &tile.animation.as_ref()?;
            Some(frames.get(frame)?.tile_id)
        };

        let accepted_style = match tile.properties.get("accepts") {
            Some(x) => AcceptedCrateStyle::Specific(CrateStyle::from_tiled_property(x)?),
            None => AcceptedCrateStyle::Any,
        };

        let pending_tex_rect = tilesheet
            .tile_rect(id)
            .ok_or_else(|| anyhow::anyhow!("could not obtain goal tile rect"))?;
        let done_tex_rect = tilesheet
            .tile_rect(
                get_frame_id(Self::DONE_FRAME)
                    .ok_or_else(|| anyhow::anyhow!("could not obtain goal DONE frame gid"))?,
            )
            .ok_or_else(|| anyhow::anyhow!("could not obtain goal DONE tile rect"))?;

        let sprite_atlas = {
            let mut sprite_atlas = SpriteAtlas::with_texture_and_frames(
                tilesheet.texture(),
                &[pending_tex_rect, done_tex_rect],
            );
            sprite_atlas.set_position(
                Vector2f::new(position.x as f32, position.y as f32).cwise_mul(grid_size),
            );
            sprite_atlas
        };

        Ok(Self {
            position,
            accepted_style,
            sprite_atlas,
        })
    }

    pub fn set_done(&mut self, val: bool) {
        self.sprite_atlas
            .set_frame(if val {
                Self::DONE_FRAME
            } else {
                Self::PENDING_FRAME
            })
            .unwrap();
    }

    pub fn is_done(&self) -> bool {
        self.sprite_atlas.current_frame() == Self::DONE_FRAME
    }

    /// Get the goal's position.
    pub fn position(&self) -> Vector2i {
        self.position
    }

    /// Get the goal's accepted style.
    pub fn accepted_style(&self) -> AcceptedCrateStyle {
        self.accepted_style
    }
}

impl<'s> Drawable for Goal<'s> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        self.sprite_atlas.draw(target, states);
    }
}
