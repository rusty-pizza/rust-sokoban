use sfml::system::Vector2i;
use tiled::map::Map;

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
        object: &tiled::objects::Object,
        map: &Map,
        tilesheet: &'s Tilesheet,
    ) -> Option<Self> {
        let position = Vector2i::new(
            (object.x / map.tile_width as f32) as i32,
            (object.y / map.tile_height as f32) as i32,
        );
        let object_tile = tilesheet.tileset().get_tile_by_gid(object.gid);
        let object_type = object_tile.and_then(|t| t.tile_type.as_deref());

        match object_type {
            Some("spawn") => Some(MapObject::Spawn { position }),
            Some("crate") => Some(MapObject::Crate(
                Crate::new(position, tilesheet, object.gid).expect("crate creation"),
            )),
            Some("goal") => Some(MapObject::Goal(
                Goal::new(position, tilesheet, object.gid).expect("goal creation"),
            )),
            _ => None,
        }
    }
}
