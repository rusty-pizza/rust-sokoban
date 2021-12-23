use sfml::{
    graphics::{Drawable, Transformable, Rect},
    system::{Vector2f, Vector2i},
};
use tiled::{tile::Gid, properties::PropertyValue};

use crate::graphics::{SpriteAtlas, Tilesheet};

use super::Direction;

/// Represents the player inside of a level.
pub struct Player<'s> {
    position: Vector2i,
    atlas: SpriteAtlas<'s>,
    direction: Direction,
}

impl Player<'_> {
    pub fn new(position: Vector2i, tilesheet: &Tilesheet) -> Option<Player> {
        let texture = tilesheet.texture();

        let get_rect = |property_name: &str| -> Option<Rect<i32>> {
            let prop = tilesheet.tileset().properties.0.get(property_name)?;
            match prop {
                PropertyValue::IntValue(x) => {
                    tilesheet.tile_rect(Gid(*x as u32 + tilesheet.tileset().first_gid.0))
                }
                _ => None
            }
        };

        let north_frame = get_rect("player_up")?;
        let south_frame = get_rect("player_down")?;
        let east_frame = get_rect("player_right")?;
        let west_frame = get_rect("player_left")?;
        let mut atlas = SpriteAtlas::with_texture_and_frames(texture, &[north_frame, south_frame, east_frame, west_frame]);

        atlas.set_position(Vector2f::new(position.x as f32, position.y as f32));
        atlas.set_scale(Vector2f::new(
            1f32 / tilesheet.tile_size().x as f32,
            1f32 / tilesheet.tile_size().y as f32,
        ));

        Some(Player { position, atlas, direction: Direction::South })
    }

    pub fn set_transform(&mut self, position: Vector2i, direction: Direction) {
        self.set_position(position);
        self.set_direction(direction);
    }

    pub fn set_position(&mut self, position: Vector2i) {
        self.position = position;
        self.atlas
            .set_position(Vector2f::new(position.x as f32, position.y as f32));
    }

    pub fn position(&self) -> Vector2i {
        self.position
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
        let direction_frame = direction as usize;
        self.atlas.set_frame(direction_frame).unwrap();
    }

    pub fn direction(&self) -> Direction {
        self.direction
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
