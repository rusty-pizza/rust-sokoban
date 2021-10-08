use std::{collections::HashMap, error::Error, fmt::Display, path::Path};

use sfml::{
    graphics::{Drawable, VertexArray},
    system::{Vector2f, Vector2i, Vector2u},
};
use tiled::{LayerData, LayerTile, Object, TiledError};

use crate::{
    quadarray::QuadArray,
    tilesheet::{Tilesheet, TilesheetLoadError},
};

#[derive(Debug, Clone, Copy)]
pub enum CrateStyle {
    Wooden,
    Red,
    Blue,
    Green,
    Metal,
}

pub enum ObjectType {
    CrateGoal,
    Crate,
}

type StyleGidMap = HashMap<u32, (ObjectType, CrateStyle)>;

pub struct Crate {
    position: Vector2i,
    style: CrateStyle,
}

pub struct Goal {
    position: Vector2i,
    style: CrateStyle,
}

type LayerTiles = Vec<LayerTile>;

pub struct Map {
    player_spawn: Vector2i,
    crates: Vec<Crate>,
    goals: Vec<Goal>,
    size: Vector2u,
    building_layer: LayerTiles,
    floor_layer: LayerTiles,
    tilesheet: Tilesheet,
    vao: VertexArray,
}

#[derive(Debug)]
pub enum MapLoadError {
    NoPlayerSpawn,
    NoGoalsOrCrates,
    NotFinite,
    InvalidLayers,
    InvalidTilesheetCount,
    TilesheetLoadError(TilesheetLoadError),
    InvalidObjectGroups,
    InvalidObject(Object),
    TiledError(TiledError),
}

impl Display for MapLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MapLoadError::NoPlayerSpawn => f.write_str("No player spawn"),
            MapLoadError::NoGoalsOrCrates => f.write_str("No goals or crates"),
            MapLoadError::NotFinite => f.write_str("Not finite"),
            MapLoadError::InvalidLayers => f.write_str("Invalid layers"),
            MapLoadError::InvalidTilesheetCount => f.write_str("Invalid tilesheet count"),
            MapLoadError::TilesheetLoadError(err) => f.write_fmt(format_args!("Tilesheet load error: {}", err)),
            MapLoadError::InvalidObjectGroups => f.write_str("Invalid object groups"),
            MapLoadError::InvalidObject(obj) => f.write_fmt(format_args!("Invalid object: {:?}", obj)),
            MapLoadError::TiledError(err) => f.write_fmt(format_args!("Tiled error: {}", err)),
        }
    }
}

impl Error for MapLoadError { }

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

impl Map {
    pub fn from_file(path: &Path, style_map: StyleGidMap) -> Result<Map, MapLoadError> {
        let data = tiled::parse_file(path)?;

        if data.infinite {
            return Err(MapLoadError::NotFinite);
        }

        if data.layers.len() > 2 {
            return Err(MapLoadError::InvalidLayers);
        }

        if data.tilesets.len() != 1 {
            return Err(MapLoadError::InvalidTilesheetCount);
        }

        let tilesheet = {
            let tileset = data.tilesets.first().unwrap().clone();
            Tilesheet::from_tileset(tileset, path.parent().expect("obtaining parent of path"))?
        };

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
            let position = Vector2i::new(object.x as i32, object.y as i32);
            if object.name == "player" {
                player_spawn = Some(position);
            } else if let Some((obj_type, crate_style)) = style_map.get(&object.gid) {
                match obj_type {
                    &ObjectType::Crate => crates.push(Crate {
                        position,
                        style: crate_style.clone(),
                    }),
                    &ObjectType::CrateGoal => goals.push(Goal {
                        position,
                        style: crate_style.clone(),
                    }),
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

    pub fn tilesheet(&self) -> &Tilesheet {
        &self.tilesheet
    }
}

impl Drawable for Map {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        let mut state = states.clone();
        state.set_texture(Some(&self.tilesheet.texture()));
        target.draw_vertex_array(&self.vao, &state);
    }
}
