//! Structures related to a sokoban level or puzzle.

// TODO: Undo history

#![allow(dead_code)]

mod error;
pub mod objects;
mod player;

use std::path::Path;

use sfml::{
    graphics::{Color, Drawable, PrimitiveType, Vertex},
    system::{Vector2f, Vector2i, Vector2u},
};
use tiled::{
    layers::{Layer, LayerData, LayerTile},
    map::Map,
    tile::Gid,
    tileset::Tileset,
};

use crate::{
    assets::AssetManager,
    graphics::{QuadMeshable, Tilesheet},
};

pub use self::error::LevelLoadError;
use self::objects::{Crate, CrateType, Goal};
pub use self::player::Player;

/// One of a level's tiles. Level tiles are inmutable because they are part of the mesh of it.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LevelTile {
    Solid,
    Hole,
    Floor,
}

/// A cardinal direction.
#[derive(Clone, Copy)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl From<Direction> for Vector2i {
    fn from(d: Direction) -> Self {
        match d {
            Direction::North => Vector2i::new(0, -1),
            Direction::South => Vector2i::new(0, 1),
            Direction::West => Vector2i::new(-1, 0),
            Direction::East => Vector2i::new(1, 0),
        }
    }
}

/// Represents a sokoban level or puzzle.
pub struct Level<'s> {
    player_spawn: Vector2i,
    crates: Vec<Crate<'s>>,
    goals: Vec<Goal<'s>>,
    size: Vector2u,
    tiles: Vec<LevelTile>,
    tilesheet: &'s Tilesheet,
    vertices: Vec<Vertex>,
    pub background_color: Color,
    player: Player<'s>,
    last_key_states: [bool; 4],
}

/// Constructors & parsing-related functions
impl<'s> Level<'s> {
    /// Load a sokoban level from a Tiled map along with a provided asset manager.
    pub fn from_map(data: &Map, assets: &'s mut AssetManager) -> Result<Level<'s>, LevelLoadError> {
        if data.infinite {
            return Err(LevelLoadError::NotFinite);
        }
        if data.tilesets.len() != 1 {
            todo!("Support for maps with multiple tilesets")
        }

        let tilesheet = {
            let tileset = data.tilesets[0].clone();
            let path = tileset.source.as_ref().unwrap().clone();
            assets.get_or_insert_asset(&path, Tilesheet::from_tileset(tileset)?)
        };

        let size = Vector2u::new(data.width, data.height);

        let (building_layer, floor_layer) = Self::get_building_and_floor_layers(&data.layers)
            .ok_or(LevelLoadError::InvalidLayers)?;

        let tiles = Self::extract_level_tiles(&building_layer, tilesheet.tileset());

        if data.object_groups.len() != 1 {
            return Err(LevelLoadError::InvalidObjectGroups);
        }

        let objects = match data.object_groups.first() {
            Some(x) if x.name == "objects" => &x.objects,
            _ => return Err(LevelLoadError::InvalidObjectGroups),
        };

        let (crates, goals, player_spawn) = {
            let mut crates = Vec::new();
            let mut goals = Vec::new();
            let mut player_spawn = None;
            for object in objects {
                let position = Vector2i::new(
                    (object.x / data.tile_width as f32) as i32,
                    (object.y / data.tile_height as f32) as i32,
                );

                let object_tile = tilesheet.tileset().get_tile_by_gid(object.gid);

                match object_tile
                    .and_then(|t| Some(t.tile_type.as_deref()))
                    .flatten()
                {
                    Some("spawn") => player_spawn = Some(position),
                    Some("crate") => crates.push(
                        Crate::new(position, &tilesheet, object.gid).expect("crate creation"),
                    ),
                    Some("goal") => {
                        let accepted_style = object
                            .properties
                            .0
                            .iter()
                            .find(|&(name, _)| name == "accepts")
                            .and_then(|(_, prop)| Some(CrateType::from_tiled_property(prop)))
                            .unwrap_or_default();

                        goals.push(
                            Goal::new(position, accepted_style, &tilesheet, object.gid)
                                .expect("goal creation"),
                        )
                    }
                    _ => return Err(LevelLoadError::InvalidObject(object.clone())),
                }
            }

            (
                crates,
                goals,
                player_spawn.ok_or(LevelLoadError::NoPlayerSpawn)?,
            )
        };

        if goals.is_empty() || crates.is_empty() {
            return Err(LevelLoadError::NoGoalsOrCrates);
        }

        let background_color = data
            .background_color
            .and_then(|c| Some(Color::rgb(c.red, c.green, c.blue)))
            .unwrap_or(Color::BLACK);

        let vertices = Self::generate_vertices(&size, &building_layer, &floor_layer, &tilesheet);

        Ok(Self {
            player_spawn,
            crates,
            goals,
            vertices,
            tiles,
            size,
            tilesheet,
            background_color,
            player: Player::new(player_spawn, tilesheet, Gid(53)).unwrap(),
            last_key_states: [false; 4],
        })
    }

    /// Loads a sokoban level from a specified path using a specified asset manager.
    pub fn from_file(path: &Path, assets: &'s mut AssetManager) -> Result<Self, LevelLoadError> {
        let map = Map::parse_file(path)?;
        Self::from_map(&map, assets)
    }

    /// Extracts the building and floor layers from the given Tiled ones.
    fn get_building_and_floor_layers(
        layers: &Vec<Layer>,
    ) -> Option<(Vec<LayerTile>, Vec<LayerTile>)> {
        let building = layers.iter().filter(|l| l.name == "building").nth(0)?;
        let floor = layers.iter().filter(|l| l.name == "floor").nth(0)?;

        match (&building.tiles, &floor.tiles) {
            (LayerData::Finite(building), LayerData::Finite(floor)) => Some((
                building.iter().flatten().copied().collect(),
                floor.iter().flatten().copied().collect(),
            )),
            _ => None,
        }
    }

    /// Extracts all level tiles from a given Tiled layer and its related tileset.
    fn extract_level_tiles(building_layer: &Vec<LayerTile>, tileset: &Tileset) -> Vec<LevelTile> {
        building_layer
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
            .collect::<Vec<_>>()
    }

    /// Generates a static level mesh and returns it.
    fn generate_vertices(
        size_in_tiles: &Vector2u,
        building_layer: &Vec<LayerTile>,
        floor_layer: &Vec<LayerTile>,
        tilesheet: &Tilesheet,
    ) -> Vec<Vertex> {
        const FLOOR_OFFSET: Vector2f = Vector2f::new(0.5f32, 0.5f32);

        let mut vertices = Vec::new();

        let iter = building_layer.iter().zip(floor_layer.iter()).enumerate();
        for (i, (b_tile, f_tile)) in iter {
            let position = Vector2f::new(
                (i % size_in_tiles.x as usize) as f32,
                (i / size_in_tiles.x as usize) as f32,
            );
            if f_tile.gid != Gid::EMPTY {
                vertices.add_quad(
                    position + FLOOR_OFFSET,
                    1f32,
                    tilesheet
                        .tile_uv(f_tile.gid)
                        .expect("obtaining floor tile UV"),
                );
            }
            if b_tile.gid != Gid::EMPTY {
                vertices.add_quad(
                    position,
                    1f32,
                    tilesheet
                        .tile_uv(b_tile.gid)
                        .expect("obtaining building tile UV"),
                );
            }
        }

        vertices
    }
}

/// Public instance functions
impl Level<'_> {
    /// Updates the level and the objects within it. Call every frame.
    pub fn update(&mut self, _delta: std::time::Duration) {
        use sfml::window::Key;
        let frame_key_states = [
            Key::W.is_pressed(),
            Key::S.is_pressed(),
            Key::A.is_pressed(),
            Key::D.is_pressed(),
        ];
        let direction = match frame_key_states {
            [true, false, false, false] => Some(Direction::North),
            [false, true, false, false] => Some(Direction::South),
            [false, false, true, false] => Some(Direction::West),
            [false, false, false, true] => Some(Direction::East),
            _ => None,
        };

        if let Some(direction) = direction {
            if self.last_key_states == [false; 4] {
                self.move_player(direction);
            }
        }
        self.last_key_states = frame_key_states;
    }

    /// Gets a level tile in a specific position.
    pub fn get_tile(&self, pos: Vector2i) -> Option<LevelTile> {
        self.tiles
            .get((pos.x + pos.y * self.size.x as i32) as usize)
            .copied()
    }

    /// Moves the player one tile onto the given direction, if possible.
    pub fn move_player(&mut self, direction: Direction) {
        let movement: Vector2i = direction.into();

        // Get all info about where the player is about to move to: Tile, crate on top of that tile,
        // tile that the crate is being pushed to, and if there is another crate where the first one
        // is being pushed to
        let cell_to_move_to = self.player.position() + movement;

        let (tile_to_move_to, crate_to_move_to_idx) = {
            let mut tile_to_move_to = self.get_tile(cell_to_move_to);
            let crate_to_move_to_idx = self
                .crates
                .iter()
                .enumerate()
                .filter(|(_idx, c)| c.position() == cell_to_move_to)
                .nth(0)
                .and_then(|(idx, _ref)| Some(idx));

            if crate_to_move_to_idx.is_some() {
                let crate_target_cell = cell_to_move_to + movement;
                if self
                    .crates
                    .iter()
                    .filter(|c| c.position() == crate_target_cell)
                    .nth(0)
                    .is_some()
                {
                    tile_to_move_to = None;
                }
            }

            (tile_to_move_to, crate_to_move_to_idx)
        };

        if crate_to_move_to_idx.is_none() && tile_to_move_to == Some(LevelTile::Floor) {
            self.player.set_position(cell_to_move_to);
        }
    }

    /// Returns the size in tiles of the level.
    pub fn size(&self) -> Vector2u {
        self.size
    }
}

impl<'s> Drawable for Level<'s> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let mut vao_rstate = states.clone();
        vao_rstate.set_texture(Some(&self.tilesheet.texture()));
        target.draw_primitives(&self.vertices, PrimitiveType::QUADS, &vao_rstate);

        for c in self.crates.iter() {
            target.draw_with_renderstates(c, &states);
        }
        for g in self.goals.iter() {
            target.draw_with_renderstates(g, &states);
        }

        target.draw_with_renderstates(&self.player, states);
    }
}
