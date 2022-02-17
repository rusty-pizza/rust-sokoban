//! Dynamic objects that are owned by the level, these being [`Crate`]s and [`Goal`]s.

#![allow(dead_code)]

use std::num::NonZeroU32;

use sfml::{
    graphics::{Drawable, Transformable},
    system::{Vector2f, Vector2i},
};
use tiled::{properties::PropertyValue, tile::Gid};

use crate::{graphics::SpriteAtlas, graphics::Tilesheet};

pub(super) mod parsing;

/// When applied to a crate, the crate's type. When applied to a goal, the crate type
/// the goal accepts.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct CrateStyle(NonZeroU32);

impl CrateStyle {
    pub(super) fn from_tiled_property(prop: &PropertyValue) -> Option<Self> {
        if let PropertyValue::IntValue(style) = prop {
            NonZeroU32::new(*style as u32).and_then(|id| Some(CrateStyle(id)))
        } else {
            None
        }
    }
}

pub enum AcceptedCrateStyle {
    Specific(CrateStyle),
    Any,
}

impl AcceptedCrateStyle {
    pub(super) fn from_tiled_property(prop: &PropertyValue) -> Self {
        CrateStyle::from_tiled_property(prop)
            .and_then(|style| Some(Self::Specific(style)))
            .unwrap_or(Self::Any)
    }
}

impl Default for AcceptedCrateStyle {
    fn default() -> Self {
        Self::Any
    }
}

/// A crate the player can move around.
pub struct Crate<'s> {
    position: Vector2i,
    sprite_atlas: SpriteAtlas<'s>,
    style: CrateStyle,
    in_hole: bool,
}

impl<'s> Crate<'s> {
    const NORMAL_FRAME: usize = 0;
    const DROPPED_FRAME: usize = 1;
    const POSITIONED_FRAME: usize = 2;
    const TRANSLUCENT_ALPHA: u8 = 150;

    pub fn new(position: Vector2i, tilesheet: &'s Tilesheet, gid: Gid) -> Option<Self> {
        let tile = tilesheet.tileset().get_tile_by_gid(gid)?;

        let crate_type = tile
            .properties
            .0
            .get("style")
            .and_then(|prop| CrateStyle::from_tiled_property(prop))?;

        let get_frame_gid = |frame: usize| -> Option<Gid> {
            let frames = &tile.animation.as_ref()?.frames;
            Some(Gid(
                frames.get(frame)?.tile_id + tilesheet.tileset().first_gid.0
            ))
        };

        let normal_tex_rect = tilesheet.tile_rect(gid)?;
        let dropped_tex_rect = tilesheet.tile_rect(get_frame_gid(Self::DROPPED_FRAME)?)?;
        let positioned_tex_rect = tilesheet.tile_rect(get_frame_gid(Self::POSITIONED_FRAME)?)?;

        let sprite_atlas = {
            let mut sprite_atlas = SpriteAtlas::with_texture_and_frames(
                tilesheet.texture(),
                &[normal_tex_rect, dropped_tex_rect, positioned_tex_rect],
            );
            sprite_atlas.set_position(Vector2f::new(position.x as f32, position.y as f32));
            sprite_atlas.set_scale(Vector2f::new(
                1f32 / tilesheet.tile_size().x as f32,
                1f32 / tilesheet.tile_size().y as f32,
            ));
            sprite_atlas
        };

        Some(Self {
            position,
            style: crate_type,
            sprite_atlas,
            in_hole: false,
        })
    }

    pub fn position(&self) -> Vector2i {
        self.position
    }

    pub fn set_position(&mut self, position: Vector2i) {
        self.position = position;
        self.sprite_atlas
            .set_position(Vector2f::new(position.x as f32, position.y as f32));
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
}

impl<'s> Drawable for Crate<'s> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        self.sprite_atlas.draw(target, &states);
    }
}

/// Indicates where a certain style of crate should be put in a level.
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
        accepted_style: AcceptedCrateStyle,
        tilesheet: &'s Tilesheet,
        gid: Gid,
    ) -> Option<Self> {
        let tile = tilesheet.tileset().get_tile_by_gid(gid)?;

        let get_frame_gid = |frame: usize| -> Option<Gid> {
            let frames = &tile.animation.as_ref()?.frames;
            Some(Gid(
                frames.get(frame)?.tile_id + tilesheet.tileset().first_gid.0
            ))
        };

        let pending_tex_rect = tilesheet.tile_rect(gid)?;
        let done_tex_rect = tilesheet.tile_rect(get_frame_gid(Self::DONE_FRAME)?)?;

        let sprite_atlas = {
            let mut sprite_atlas = SpriteAtlas::with_texture_and_frames(
                tilesheet.texture(),
                &[pending_tex_rect, done_tex_rect],
            );
            sprite_atlas.set_position(Vector2f::new(position.x as f32, position.y as f32));
            sprite_atlas.set_scale(Vector2f::new(
                1f32 / tilesheet.tile_size().x as f32,
                1f32 / tilesheet.tile_size().y as f32,
            ));
            sprite_atlas
        };

        Some(Self {
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
