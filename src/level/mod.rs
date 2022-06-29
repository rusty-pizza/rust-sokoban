//! Structures related to a sokoban level or puzzle.

#![allow(dead_code)]

mod action;
use action::*;
mod error;
pub mod objects;
mod player;
pub mod tilemap;

use rand::{prelude::SliceRandom, thread_rng};
use sfml::{
    audio::{Sound, SoundSource},
    graphics::{Color, Drawable, PrimitiveType, Vertex},
    system::{Vector2f, Vector2i, Vector2u},
    window::{Event, Key},
};
use tiled::{
    layers::{Layer, LayerData, LayerTile},
    map::Map,
    tile::Gid,
};

use crate::{
    context::Context,
    graphics::{QuadMeshable, Tilesheet},
};

pub use self::error::LevelLoadError;
pub use self::player::Player;
use self::{
    objects::{Crate, Goal},
    tilemap::{LevelTile, Tilemap},
};

/// A cardinal direction.
#[derive(Clone, Copy)]
#[repr(usize)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Direction {
    pub fn inverse(self) -> Self {
        match self {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        }
    }
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

fn play_move_sound(context: Context) {
    let buf_to_use = context
        .assets
        .walk_sounds
        .choose(&mut thread_rng())
        .expect("No walk sounds to play");

    let mut sound = Sound::with_buffer(buf_to_use);
    sound.set_volume(40.);
    sound.play();
    context.sound.add_sound(sound);
}

fn play_undo_sound(context: Context) {
    let buf_to_use = context
        .assets
        .undo_sounds
        .choose(&mut thread_rng())
        .expect("No walk sounds to play");

    let mut sound = Sound::with_buffer(buf_to_use);
    sound.set_volume(40.);
    sound.play();
    context.sound.add_sound(sound);
}
/// Represents a sokoban level or puzzle.
pub struct Level<'s> {
    player_spawn: Vector2i,
    crates: Vec<Crate<'s>>,
    goals: Vec<Goal<'s>>,
    tilemap: Tilemap,
    tilesheet: &'s Tilesheet,
    vertices: Vec<Vertex>,
    pub background_color: Color,
    player: Player<'s>,
    undo_history: Vec<Action>,
}

/// Constructors & parsing-related functions
impl<'s> Level<'s> {
    /// Load a sokoban level from a Tiled map and its tilesheet.
    pub fn from_map(map: &Map, tilesheet: &'s Tilesheet) -> Result<Level<'s>, LevelLoadError> {
        if map.infinite {
            return Err(LevelLoadError::NotFinite);
        }
        if map.tilesets.len() != 1 {
            todo!("Support for maps with multiple tilesets")
        }

        let size = Vector2u::new(map.width, map.height);

        let (building_layer, floor_layer) = Self::get_building_and_floor_layers(&map.layers)
            .ok_or(LevelLoadError::InvalidLayers)?;

        let tilemap = Tilemap::from_tiled_layer(size, &building_layer, tilesheet.tileset());

        if map.object_groups.len() != 1 {
            return Err(LevelLoadError::InvalidObjectGroups);
        }

        let (crates, goals, player_spawn) = {
            let mut crates = Vec::new();
            let mut goals = Vec::new();
            let mut player_spawn = None;

            let objects = &map.object_groups[0].objects;
            for object in objects {
                use objects::parsing::MapObject::{self, *};

                match MapObject::from_tiled_object(object, map, tilesheet) {
                    Some(Spawn { position }) => player_spawn = Some(position),
                    Some(Crate(c)) => crates.push(c),
                    Some(Goal(g)) => goals.push(g),

                    None => return Err(LevelLoadError::InvalidObject(object.clone())),
                }
            }

            if goals.is_empty() || crates.is_empty() {
                return Err(LevelLoadError::NoGoalsOrCrates);
            }

            (
                crates,
                goals,
                player_spawn.ok_or(LevelLoadError::NoPlayerSpawn)?,
            )
        };

        let player = Player::new(player_spawn, tilesheet).expect("constructing player");

        let background_color = map
            .background_color
            .and_then(|c| Some(Color::rgb(c.red, c.green, c.blue)))
            .unwrap_or(Color::BLACK);

        let vertices = Self::generate_vertices(&size, &building_layer, &floor_layer, &tilesheet);

        Ok(Self {
            player_spawn,
            crates,
            goals,
            vertices,
            tilemap,
            tilesheet,
            background_color,
            player,
            undo_history: vec![],
        })
    }

    /// Extracts the building and floor layers from the given Tiled ones.
    fn get_building_and_floor_layers(
        layers: &Vec<Layer>,
    ) -> Option<(Vec<LayerTile>, Vec<LayerTile>)> {
        let building = layers.iter().find(|l| l.name == "building")?;
        let floor = layers.iter().find(|l| l.name == "floor")?;

        match (&building.tiles, &floor.tiles) {
            (LayerData::Finite(building), LayerData::Finite(floor)) => Some((
                building.iter().flatten().copied().collect(),
                floor.iter().flatten().copied().collect(),
            )),
            _ => None,
        }
    }

    /// Generates a static level mesh and returns it.
    fn generate_vertices(
        size_in_tiles: &Vector2u,
        building_layer: &Vec<LayerTile>,
        floor_layer: &Vec<LayerTile>,
        tilesheet: &Tilesheet,
    ) -> Vec<Vertex> {
        const FLOOR_OFFSET: Vector2f = Vector2f::new(0.5f32, 0.5f32);
        const TILE_DILATION: f32 = 0.01;

        let mut vertices = Vec::new();

        let iter = building_layer.iter().zip(floor_layer.iter()).enumerate();
        for (i, (b_tile, f_tile)) in iter {
            let position = Vector2f::new(
                (i % size_in_tiles.x as usize) as f32,
                (i / size_in_tiles.x as usize) as f32,
            );
            if f_tile.gid != Gid::EMPTY {
                vertices.add_quad(
                    position + FLOOR_OFFSET - TILE_DILATION,
                    1f32 + TILE_DILATION * 2.,
                    tilesheet
                        .tile_uv(f_tile.gid)
                        .expect("obtaining floor tile UV"),
                );
            }
            if b_tile.gid != Gid::EMPTY {
                vertices.add_quad(
                    position - TILE_DILATION,
                    1f32 + TILE_DILATION * 2.,
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
    /// The tilemap associated to the level.
    pub fn tilemap(&self) -> &Tilemap {
        &self.tilemap
    }

    /// Returns whether all the crates are in goals or not.
    pub fn is_won(&self) -> bool {
        self.goals.iter().all(|g| g.is_done())
    }

    pub fn handle_event(&mut self, context: Context, event: Event) {
        match event {
            Event::KeyPressed { code: Key::A, .. } => {
                self.move_player(Direction::West, context);
            }
            Event::KeyPressed { code: Key::W, .. } => {
                self.move_player(Direction::North, context);
            }
            Event::KeyPressed { code: Key::S, .. } => {
                self.move_player(Direction::South, context);
            }
            Event::KeyPressed { code: Key::D, .. } => {
                self.move_player(Direction::East, context);
            }
            Event::KeyPressed { code: Key::Q, .. } => {
                self.undo(context);
            }
            _ => (),
        }
    }

    pub fn undo(&mut self, context: Context) {
        if let Some(m) = self.undo_history.pop() {
            m.apply(self).expect("couldn't undo move");
            play_undo_sound(context);
        }
    }

    /// Updates the level and the objects within it. Call every frame.
    pub fn update(&mut self, _context: Context, _delta: std::time::Duration) {
        self.update_crate_opacity();
    }

    fn update_crate_opacity(&mut self) {
        fn get_crates_on_top(crates: &[Crate]) -> Vec<usize> {
            let mut crates_on_top = Vec::new();
            for c in 0..crates.len() {
                if crates[c].in_hole() {
                    for c_on_top in 0..crates.len() {
                        if c != c_on_top {
                            if crates[c_on_top].position() == crates[c].position() {
                                crates_on_top.push(c_on_top);
                            }
                        }
                    }
                }
            }
            crates_on_top
        }

        self.crates.iter_mut().for_each(|c| {
            c.set_opaque(true);
        });

        get_crates_on_top(&self.crates).into_iter().for_each(|c| {
            self.crates[c].set_opaque(false);
        });

        self.goals.iter_mut().for_each(|g| g.set_done(false));
        self.crates.iter_mut().for_each(|c| {
            if !c.in_hole() {
                c.set_is_positioned(false)
            }
        });

        self.goals.iter_mut().for_each(|g| {
            self.crates
                .iter_mut()
                .filter(|c| c.position() == g.position())
                .nth(0)
                .map(|c| {
                    g.set_done(true);
                    c.set_is_positioned(true);
                });
        })
    }

    /// Moves the player one tile onto the given direction, if possible.
    pub fn move_player(&mut self, direction: Direction, context: Context) {
        let action = Action::Push {
            direction,
            look_direction: direction,
        };
        if let Ok(undo) = action.apply(self) {
            self.undo_history.push(undo);
            play_move_sound(context);
        }
    }

    /// Returns true if there is a solid tile or crate in the given position.
    pub fn is_cell_obstructed(&self, position: Vector2i) -> bool {
        let cell_tile_is_solid = self.tilemap.get_tile(position) == Some(LevelTile::Solid);
        let cell_has_crate = self
            .crates
            .iter()
            .any(|c| c.position() == position && !c.in_hole());
        cell_tile_is_solid || cell_has_crate
    }

    /// Returns whether a given cell can be walked over or not, regardless of whether there is a
    /// movable crate in that position or not.
    pub fn is_cell_walkable(&self, position: Vector2i) -> bool {
        let tile = self.tilemap.get_tile(position);
        match tile {
            Some(LevelTile::Hole) => {
                let is_there_walkable_crate = self
                    .crates
                    .iter()
                    .any(|c| c.position() == position && c.in_hole());
                is_there_walkable_crate
            }
            Some(LevelTile::Floor) => true,
            Some(LevelTile::Solid) | None => false,
        }
    }
}

impl<'s> Drawable for Level<'s> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let mut level_rstate = states.clone();
        level_rstate.set_texture(Some(&self.tilesheet.texture()));
        target.draw_primitives(&self.vertices, PrimitiveType::QUADS, &level_rstate);

        // draw crates in holes (underground) first
        self.crates
            .iter()
            .filter(|c| c.in_hole())
            .for_each(|c| target.draw_with_renderstates(c, &states));

        // then draw the ones on top of the ground
        self.crates
            .iter()
            .filter(|c| !c.in_hole())
            .for_each(|c| target.draw_with_renderstates(c, &states));

        self.goals
            .iter()
            .for_each(|g| target.draw_with_renderstates(g, &states));

        target.draw_with_renderstates(&self.player, states);
    }
}
