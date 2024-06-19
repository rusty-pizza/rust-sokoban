use sfml::system::{Vector2f, Vector2i};

use crate::graphics::Tilesheet;

use super::{Crate, Goal};

pub enum MapObject<'s> {
    Spawn { position: Vector2i },
    Crate(Crate<'s>),
    Goal(Goal<'s>),
}

impl<'s> MapObject<'s> {
    /// Parses a Tiled map object into a [`MapObject`] if it is valid.
    pub fn from_tiled_object(
        object: &tiled::Object,
        map: &tiled::Map,
        tilesheet: &'s Tilesheet,
    ) -> Option<Self> {
        let position = Vector2i::new(
            (object.x / map.tile_width as f32) as i32,
            (object.y / map.tile_height as f32) as i32,
        );
        let tile_id = object.tile_data().unwrap().id();
        let object_tile = tilesheet.tileset().get_tile(tile_id);
        let object_type = object_tile.as_ref().and_then(|t| t.user_type.as_deref());

        let grid_size = Vector2f::new(map.tile_width as f32, map.tile_height as f32);

        match object_type {
            Some("spawn") => Some(MapObject::Spawn { position }),
            Some("crate") => Some(MapObject::Crate(
                Crate::new(position, tilesheet, tile_id, grid_size).expect("crate creation"),
            )),
            Some("goal") => Some(MapObject::Goal(
                Goal::new(position, tilesheet, tile_id, grid_size).expect("goal creation"),
            )),
            _ => None,
        }
    }
}
