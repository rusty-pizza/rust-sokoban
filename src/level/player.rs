use sfml::{
    graphics::{Drawable, Transformable},
    system::{Vector2f, Vector2i},
};
use tiled::tile::Gid;

use crate::graphics::{SpriteAtlas, Tilesheet};

// TODO: Reflect player direction & animation on sprite

/// Represents the player inside of a level.
pub struct Player<'s> {
    position: Vector2i,
    atlas: SpriteAtlas<'s>,
}

impl Player<'_> {
    pub fn new(position: Vector2i, tilesheet: &Tilesheet, gid: Gid) -> Option<Player> {
        let texture = tilesheet.texture();
        let rect = tilesheet.tile_rect(gid)?;
        let mut atlas = SpriteAtlas::with_texture_and_frames(texture, &[rect]);
        atlas.set_position(Vector2f::new(position.x as f32, position.y as f32));
        atlas.set_scale(Vector2f::new(
            1f32 / tilesheet.tile_size().x as f32,
            1f32 / tilesheet.tile_size().y as f32,
        ));

        Some(Player { position, atlas })
    }

    pub fn set_position(&mut self, position: Vector2i) {
        self.position = position;
        self.atlas
            .set_position(Vector2f::new(position.x as f32, position.y as f32));
    }

    pub fn position(&self) -> Vector2i {
        self.position
    }
}

impl Drawable for Player<'_> {
    fn draw<'s: 'shader, 'texture, 'shader, 'shader_texture>(
        &'s self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        target.draw_with_renderstates(&self.atlas, states);
    }
}
