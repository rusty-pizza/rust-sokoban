//! Dynamic objects that are owned by the level, such as crates and goals.

#![allow(dead_code)]

use std::num::NonZeroU32;

use sfml::{
    graphics::{Drawable, Sprite, Transformable},
    system::{Vector2f, Vector2i},
};
use tiled::{properties::PropertyValue, tile::Gid};

use crate::{graphics::SpriteAtlas, graphics::Tilesheet};

/// When applied to a crate, the crate's type. When applied to a goal, the crate type
/// the goal accepts.
pub enum CrateType {
    WithId(NonZeroU32),
    Any,
}

impl Default for CrateType {
    fn default() -> Self {
        CrateType::Any
    }
}

impl CrateType {
    pub(super) fn from_tiled_property(prop: &PropertyValue) -> Self {
        match prop {
            PropertyValue::IntValue(style) => match NonZeroU32::new(*style as u32) {
                Some(x) => CrateType::WithId(x),
                None => CrateType::Any,
            },
            _ => CrateType::Any,
        }
    }
}

/// A crate the player can move around.
pub struct Crate<'s> {
    position: Vector2i,
    sprite_atlas: SpriteAtlas<'s>,
    crate_type: CrateType,
}

impl<'s> Crate<'s> {
    const NORMAL_FRAME: usize = 0;
    const DROPPED_FRAME: usize = 1;
    const POSITIONED_FRAME: usize = 2;

    pub fn new(position: Vector2i, tilesheet: &'s Tilesheet, gid: Gid) -> Option<Self> {
        let tile = tilesheet.tileset().get_tile_by_gid(gid)?;

        let crate_type = tile
            .properties
            .0
            .iter()
            .find(|&(name, _)| name == "style")
            .and_then(|(_, prop)| Some(CrateType::from_tiled_property(prop)))
            .unwrap_or_default();

        let normal_tex_rect = tilesheet.tile_rect(gid)?;
        let dropped_tex_rect = tilesheet.tile_rect(Gid(tile
            .animation
            .as_ref()?
            .frames
            .get(Self::DROPPED_FRAME)?
            .tile_id
            + tilesheet.tileset().first_gid.0))?;
        let positioned_tex_rect = tilesheet.tile_rect(Gid(tile
            .animation
            .as_ref()?
            .frames
            .get(Self::POSITIONED_FRAME)?
            .tile_id
            + tilesheet.tileset().first_gid.0))?;

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
            crate_type,
            sprite_atlas,
        })
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

/// Indicates where a specific type of crate should be put in a level.
pub struct Goal<'s> {
    position: Vector2i,
    accepted_type: CrateType,
    sprite: Sprite<'s>,
}

impl<'s> Goal<'s> {
    pub fn new(
        position: Vector2i,
        accepted_style: CrateType,
        tilesheet: &'s Tilesheet,
        gid: Gid,
    ) -> Option<Self> {
        tilesheet.tile_sprite(gid).map(|mut sprite| {
            sprite.set_position(Vector2f::new(position.x as f32, position.y as f32));
            sprite.set_scale({
                let rect = sprite.texture_rect();
                Vector2f::new(1f32 / rect.width as f32, 1f32 / rect.height as f32)
            });
            Self {
                position,
                accepted_type: accepted_style,
                sprite,
            }
        })
    }
}

impl<'s> Drawable for Goal<'s> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        self.sprite.draw(target, states);
    }
}
