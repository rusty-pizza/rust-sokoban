use sfml::system::{Vector2i, Vector2u};
use tiled::{layers::LayerTile, tile::Gid, tileset::Tileset};

/// One of a level's tiles. Level tiles are inmutable because they are part of the mesh of it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LevelTile {
    Solid,
    Hole,
    Floor,
}

/// A bidimensional array of level tiles.
pub struct Tilemap {
    size: Vector2u,
    tiles: Vec<LevelTile>,
}

impl Tilemap {
    /// Extracts a Tilemap from a given Tiled layer, its related tileset and size.
    pub fn from_tiled_layer(
        size: Vector2u,
        building_layer: &Vec<LayerTile>,
        tileset: &Tileset,
    ) -> Self {
        let tiles = building_layer
            .iter()
            .map(|tile| {
                if tile.gid == Gid::EMPTY {
                    return LevelTile::Floor;
                }

                let tile_data = tileset.get_tile_by_gid(tile.gid);

                match tile_data.and_then(|t| t.tile_type.as_deref()) {
                    Some("solid") => LevelTile::Solid,
                    Some("hole") => LevelTile::Hole,
                    _ => LevelTile::Floor,
                }
            })
            .collect::<Vec<_>>();

        Self { size, tiles }
    }

    /// The bidimensional size of this tilemap, in tiles.
    pub fn size(&self) -> Vector2u {
        self.size
    }

    /// Obtains a tile from the tilemap in a given position, if it exists.
    pub fn get_tile(&self, pos: Vector2i) -> Option<LevelTile> {
        self.tiles
            .get((pos.x + pos.y * self.size.x as i32) as usize)
            .copied()
    }
}
