use std::collections::HashMap;

use sfml::{
    graphics::{Drawable, Sprite, Transformable, VertexArray},
    system::{Vector2f, Vector2i, Vector2u},
};
use tiled::{LayerData, LayerTile, Object, TiledError};

use crate::{
    quadarray::QuadArray,
    tilesheet::{Tilesheet, TilesheetLoadError},
};

use thiserror::Error;

#[derive(Debug, Clone, Copy)]
pub enum CrateStyle {
    Wooden,
    Red,
    Blue,
    Green,
    Metal,
}

#[derive(Debug, Clone, Copy)]
pub enum ObjectType {
    CrateGoal,
    Crate,
}

type StyleGidMap = HashMap<u32, (ObjectType, CrateStyle)>;

// For now, Crate and Goal are practically the same, but this will change once more complexity is
// added
pub struct Crate<'s> {
    position: Vector2i,
    style: CrateStyle,
    sprite: Sprite<'s>,
}

impl<'s> Crate<'s> {
    fn new(
        position: Vector2i,
        style: CrateStyle,
        tilesheet: &'s Tilesheet,
        gid: u32,
    ) -> Option<Self> {
        tilesheet.tile_sprite(gid).map(|mut sprite| {
            sprite.set_position(Vector2f::new(position.x as f32, position.y as f32));
            sprite.set_scale({
                let rect = sprite.texture_rect();
                Vector2f::new(1f32 / rect.width as f32, 1f32 / rect.height as f32)
            });
            Self {
                position,
                style,
                sprite,
            }
        })
    }
}

impl<'s> Drawable for Crate<'s> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        self.sprite.draw(target, states);
    }
}

pub struct Goal<'s> {
    position: Vector2i,
    style: CrateStyle,
    sprite: Sprite<'s>,
}

impl<'s> Goal<'s> {
    fn new(
        position: Vector2i,
        style: CrateStyle,
        tilesheet: &'s Tilesheet,
        gid: u32,
    ) -> Option<Self> {
        tilesheet.tile_sprite(gid).map(|mut sprite| {
            sprite.set_position(Vector2f::new(position.x as f32, position.y as f32));
            sprite.set_scale({
                let rect = sprite.texture_rect();
                Vector2f::new(1f32 / rect.width as f32, 1f32 / rect.height as f32)
            });
            Self {
                position,
                style,
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

type LayerTiles = Vec<LayerTile>;

pub struct Level<'s> {
    player_spawn: Vector2i,
    crates: Vec<Crate<'s>>,
    goals: Vec<Goal<'s>>,
    size: Vector2u,
    building_layer: LayerTiles,
    floor_layer: LayerTiles,
    tilesheet: &'s Tilesheet,
    vao: VertexArray,
}

#[derive(Debug, Error)]
pub enum MapLoadError {
    #[error("No player spawn")]
    NoPlayerSpawn,
    #[error("No goals or crates")]
    NoGoalsOrCrates,
    #[error("Map not finite")]
    NotFinite,
    #[error("Invalid layers")]
    InvalidLayers,
    #[error("Invalid tilesheet count")]
    InvalidTilesheetCount,
    #[error("Tilesheet load error: {0}")]
    TilesheetLoadError(TilesheetLoadError),
    #[error("Invalid object groups")]
    InvalidObjectGroups,
    #[error("Invalid object: {0:?}")]
    InvalidObject(Object),
    #[error("Tiled error: {0}")]
    TiledError(TiledError),
}

impl From<TiledError> for MapLoadError {
    fn from(x: TiledError) -> Self {
        Self::TiledError(x)
    }
}

impl From<TilesheetLoadError> for MapLoadError {
    fn from(x: TilesheetLoadError) -> Self {
        Self::TilesheetLoadError(x)
    }
}

impl<'s> Level<'s> {
    pub fn new(
        data: &tiled::Map,
        tilesheet: &'s Tilesheet,
        style_map: StyleGidMap,
    ) -> Result<Level<'s>, MapLoadError> {
        if data.infinite {
            return Err(MapLoadError::NotFinite);
        }

        if data.layers.len() > 2 {
            return Err(MapLoadError::InvalidLayers);
        }

        if data.tilesets.len() != 1 {
            return Err(MapLoadError::InvalidTilesheetCount);
        }

        let size = Vector2u::new(data.width, data.height);

        let (building_layer, floor_layer) = {
            let building = data.layers.iter().filter(|l| l.name == "building").nth(0);
            let floor = data.layers.iter().filter(|l| l.name == "floor").nth(0);

            match (
                building.and_then(|x| Some(&x.tiles)),
                floor.and_then(|x| Some(&x.tiles)),
            ) {
                (Some(LayerData::Finite(building)), Some(LayerData::Finite(floor))) => (
                    building.iter().flatten().copied().collect(),
                    floor.iter().flatten().copied().collect(),
                ),
                _ => return Err(MapLoadError::InvalidLayers),
            }
        };

        if data.object_groups.len() != 1 {
            return Err(MapLoadError::InvalidObjectGroups);
        }

        let object_group = match data.object_groups.first() {
            Some(x) if x.name == "objects" => x,
            _ => return Err(MapLoadError::InvalidObjectGroups),
        };

        let mut crates = Vec::new();
        let mut goals = Vec::new();
        let mut player_spawn = None;
        for object in object_group.objects.iter() {
            let position = Vector2i::new(
                (object.x / data.tile_width as f32) as i32,
                (object.y / data.tile_height as f32) as i32 - 1,
            );
            if object.name == "player" {
                player_spawn = Some(position);
            } else if let Some((obj_type, crate_style)) = style_map.get(&object.gid) {
                match obj_type {
                    &ObjectType::Crate => crates.push(
                        Crate::new(position, *crate_style, &tilesheet, object.gid)
                            .expect("crate creation"),
                    ),
                    &ObjectType::CrateGoal => goals.push(
                        Goal::new(position, *crate_style, &tilesheet, object.gid)
                            .expect("goal creation"),
                    ),
                }
            } else {
                return Err(MapLoadError::InvalidObject(object.clone()));
            }
        }

        let player_spawn = match player_spawn {
            Some(spawn) => spawn,
            _ => return Err(MapLoadError::NoPlayerSpawn),
        };

        if goals.is_empty() || crates.is_empty() {
            return Err(MapLoadError::NoGoalsOrCrates);
        }

        Ok(Self {
            player_spawn,
            crates,
            goals,
            vao: Self::generate_vao(&size, &building_layer, &floor_layer, &tilesheet),
            building_layer,
            floor_layer,
            size,
            tilesheet,
        })
    }

    fn generate_vao(
        size_in_tiles: &Vector2u,
        building_layer: &LayerTiles,
        floor_layer: &LayerTiles,
        tilesheet: &Tilesheet,
    ) -> VertexArray {
        const FLOOR_OFFSET: Vector2f = Vector2f::new(0.5f32, 0.5f32);

        let mut quads = QuadArray::new((size_in_tiles.x * size_in_tiles.y) as usize);

        let iter = building_layer.iter().zip(floor_layer.iter()).enumerate();
        for (i, (b_tile, f_tile)) in iter {
            let position = Vector2f::new(
                (i % size_in_tiles.x as usize) as f32,
                (i / size_in_tiles.x as usize) as f32,
            );
            if f_tile.gid > 0 {
                quads.add_quad(
                    position + FLOOR_OFFSET,
                    1f32,
                    tilesheet
                        .tile_uv(f_tile.gid)
                        .expect("obtaining floor tile UV"),
                );
            }
            if b_tile.gid > 0 {
                quads.add_quad(
                    position,
                    1f32,
                    tilesheet
                        .tile_uv(b_tile.gid)
                        .expect("obtaining building tile UV"),
                );
            }
        }

        quads.result()
    }

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
        target.draw_vertex_array(&self.vao, &vao_rstate);

        for c in self.crates.iter() {
            target.draw_with_renderstates(c, &states);
        }
        for g in self.goals.iter() {
            target.draw_with_renderstates(g, &states);
        }
    }
}
