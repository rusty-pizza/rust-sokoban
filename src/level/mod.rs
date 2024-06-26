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
    graphics::{Color, Drawable, PrimitiveType, Transform, Vertex},
    system::{Vector2f, Vector2i, Vector2u},
    window::{Event, Key},
};
use tiled::{LayerTileData, Map};

use crate::{
    context::Context,
    graphics::{QuadMeshable, Tilesheet},
    ui::{get_ui_obj_from_tiled_obj, UiObject},
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

fn play_move_sound(context: &mut Context) {
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

fn play_undo_sound(context: &mut Context) {
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
#[derive(Clone)]
pub struct Level<'s> {
    overlay: Vec<Box<dyn UiObject<'s> + 's>>,
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
    pub fn from_map(map: &Map, ctx: &Context<'s>) -> Result<Level<'s>, LevelLoadError> {
        if map.infinite() {
            return Err(LevelLoadError::NotFinite);
        }

        let assets = ctx.assets;

        let size = Vector2u::new(map.width, map.height);

        let (building_layer, floor_layer) =
            Self::get_building_and_floor_layers(map).ok_or(LevelLoadError::InvalidLayers)?;

        let tilemap = Tilemap::from_tiled_layer(size, &building_layer, assets.tilesheet.tileset());

        let object_group = map
            .layers()
            .filter_map(|layer| layer.as_object_layer())
            .next()
            .unwrap();

        let (crates, goals, player_spawn) = {
            let mut crates = Vec::new();
            let mut goals = Vec::new();
            let mut player_spawn = None;

            for object in object_group.objects() {
                use objects::parsing::MapObject::{self, *};

                match MapObject::from_tiled_object(&object, map, &assets.tilesheet) {
                    Some(Spawn { position }) => player_spawn = Some(position),
                    Some(Crate(c)) => crates.push(c),
                    Some(Goal(g)) => goals.push(g),

                    None => return Err(LevelLoadError::InvalidObject((*object).clone())),
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

        let grid_size = Vector2f::new(map.tile_width as f32, map.tile_height as f32);
        let player =
            Player::new(player_spawn, &assets.tilesheet, grid_size).expect("constructing player");

        let background_color = map
            .background_color
            .map(|c| Color::rgb(c.red, c.green, c.blue))
            .unwrap_or(Color::BLACK);

        let vertices = Self::generate_vertices(
            &size,
            &building_layer,
            &floor_layer,
            &assets.tilesheet,
            grid_size,
        );

        let overlay = map
            .layers()
            .find(|o| o.name == "overlay")
            .map_or(vec![], |o| {
                o.as_object_layer()
                    .unwrap()
                    .objects()
                    .map(|object| get_ui_obj_from_tiled_obj(ctx, &object).unwrap())
                    .collect()
            });

        Ok(Self {
            overlay,
            player_spawn,
            crates,
            goals,
            vertices,
            tilemap,
            tilesheet: &assets.tilesheet,
            background_color,
            player,
            undo_history: vec![],
        })
    }

    /// Extracts the building and floor layers from the given Tiled ones.
    fn get_building_and_floor_layers(
        map: &Map,
    ) -> Option<(Vec<Option<LayerTileData>>, Vec<Option<LayerTileData>>)> {
        let building = map
            .layers()
            .find(|l| l.name == "building")?
            .as_tile_layer()?;
        let floor = map.layers().find(|l| l.name == "floor")?.as_tile_layer()?;

        let mut building_tiles = Vec::new();
        let mut floor_tiles = Vec::new();

        for y in 0..building.height()? as i32 {
            for x in 0..building.width()? as i32 {
                let building_tile = building.get_tile(x, y).as_deref().cloned();
                let floor_tile = floor.get_tile(x, y).as_deref().cloned();
                building_tiles.push(building_tile);
                floor_tiles.push(floor_tile);
            }
        }

        Some((building_tiles, floor_tiles))
    }

    /// Generates a static level mesh and returns it.
    fn generate_vertices(
        size_in_tiles: &Vector2u,
        building_layer: &[Option<LayerTileData>],
        floor_layer: &[Option<LayerTileData>],
        tilesheet: &Tilesheet,
        grid_size: Vector2f,
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
            if let Some(f_tile) = f_tile {
                vertices.add_quad(
                    (position + FLOOR_OFFSET - Vector2f::new(TILE_DILATION, TILE_DILATION))
                        .cwise_mul(grid_size),
                    grid_size * (1f32 + TILE_DILATION * 2.),
                    tilesheet
                        .tile_uv(f_tile.id())
                        .expect("obtaining floor tile UV"),
                );
            }
            if let Some(b_tile) = b_tile {
                vertices.add_quad(
                    (position - Vector2f::new(TILE_DILATION, TILE_DILATION)).cwise_mul(grid_size),
                    grid_size * (1f32 + TILE_DILATION * 2.),
                    tilesheet
                        .tile_uv(b_tile.id())
                        .expect("obtaining building tile UV"),
                );
            }
        }

        vertices
    }

    pub fn tilesheet(&self) -> &Tilesheet {
        self.tilesheet
    }
}

/// Public instance functions
impl Level<'_> {
    pub fn action_count(&self) -> usize {
        self.undo_history.len()
    }

    /// The tilemap associated to the level.
    pub fn tilemap(&self) -> &Tilemap {
        &self.tilemap
    }

    /// Returns whether all the crates are in goals or not.
    pub fn is_won(&self) -> bool {
        self.goals.iter().all(|g| g.is_done())
    }

    pub fn handle_event(&mut self, context: &mut Context, event: Event) {
        match event {
            Event::KeyPressed { code: Key::A, .. }
            | Event::KeyPressed {
                code: Key::Left, ..
            } => {
                self.move_player(Direction::West, context);
            }
            Event::KeyPressed { code: Key::W, .. } | Event::KeyPressed { code: Key::Up, .. } => {
                self.move_player(Direction::North, context);
            }
            Event::KeyPressed { code: Key::S, .. }
            | Event::KeyPressed {
                code: Key::Down, ..
            } => {
                self.move_player(Direction::South, context);
            }
            Event::KeyPressed { code: Key::D, .. }
            | Event::KeyPressed {
                code: Key::Right, ..
            } => {
                self.move_player(Direction::East, context);
            }
            Event::KeyPressed { code: Key::Q, .. } => {
                self.undo(context);
            }
            _ => (),
        }
    }

    pub fn undo(&mut self, context: &mut Context) {
        if let Some(m) = self.undo_history.pop() {
            m.apply(self).expect("couldn't undo move");
            play_undo_sound(context);
        }
    }

    /// Updates the level and the objects within it. Call every frame.
    pub fn update(&mut self, _context: &mut Context, _delta: std::time::Duration) {
        self.update_crate_opacity();
    }

    fn update_crate_opacity(&mut self) {
        fn get_crates_on_top(crates: &[Crate]) -> Vec<usize> {
            let mut crates_on_top = Vec::new();
            for c in 0..crates.len() {
                if crates[c].in_hole() {
                    for c_on_top in 0..crates.len() {
                        if c != c_on_top && crates[c_on_top].position() == crates[c].position() {
                            crates_on_top.push(c_on_top);
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
            if let Some(c) = self.crates.iter_mut().find(|c| {
                c.position() == g.position()
                    && !c.in_hole()
                    && g.accepted_style().accepts(c.style())
            }) {
                g.set_done(true);
                c.set_is_positioned(true);
            };
        })
    }

    /// Moves the player one tile onto the given direction, if possible.
    pub fn move_player(&mut self, direction: Direction, context: &mut Context) {
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
        let mut level_rstate = *states;
        level_rstate.set_texture(Some(self.tilesheet.texture()));
        target.draw_primitives(&self.vertices, PrimitiveType::QUADS, &level_rstate);

        // draw crates in holes (underground) first
        self.crates
            .iter()
            .filter(|c| c.in_hole())
            .for_each(|c| target.draw_with_renderstates(c, states));

        // then draw the ones on top of the ground
        self.crates
            .iter()
            .filter(|c| !c.in_hole())
            .for_each(|c| target.draw_with_renderstates(c, states));

        self.goals
            .iter()
            .for_each(|g| target.draw_with_renderstates(g, states));

        target.draw_with_renderstates(&self.player, states);

        for element in self.overlay.iter() {
            target.draw_with_renderstates(element.as_drawable(), states);
        }
    }
}

pub fn camera_transform(
    window_size: Vector2u,
    map_size: Vector2u,
    vertical_padding: f32,
) -> Transform {
    let map_size = Vector2f::new(map_size.x as f32, map_size.y as f32 + vertical_padding);
    let window_size = Vector2f::new(window_size.x as f32, window_size.y as f32);
    let viewport_size = Vector2f::new(window_size.x, window_size.y);

    let scale_factors = map_size.cwise_div(viewport_size);
    let map_scale = if scale_factors.x > scale_factors.y {
        scale_factors.x
    } else {
        scale_factors.y
    };
    let map_px_size = map_size / map_scale;

    let mut x = Transform::IDENTITY;
    x.scale_with_center(map_scale, map_scale, 0f32, 0f32);
    x.translate(
        (map_px_size.x - viewport_size.x) / 2f32 + (viewport_size.x - window_size.x) / 2f32,
        (map_px_size.y - viewport_size.y) / 2f32 + (viewport_size.y - window_size.y) / 2f32,
    );
    let tile = map_px_size.y / map_size.y;
    x.translate(0., -tile * vertical_padding / 2.);
    x.inverse()
}
