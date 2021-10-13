mod objects;

use std::path::Path;

use sfml::{
    graphics::{Color, Drawable, VertexArray},
    system::{Vector2f, Vector2i, Vector2u},
};
use tiled::{
    error::TiledError,
    layers::{LayerData, LayerTile},
    map::Map,
    tile::Gid,
    tileset::Tileset,
};

use crate::{
    asset_manager::AssetManager,
    quadarray::QuadArray,
    tilesheet::{Tilesheet, TilesheetLoadError},
};

use thiserror::Error;

use self::objects::{Crate, CrateType, Goal};

enum Floor {
    Solid,
    Hole,
    Walkable,
}

pub struct Level<'s> {
    player_spawn: Vector2i,
    crates: Vec<Crate<'s>>,
    goals: Vec<Goal<'s>>,
    size: Vector2u,
    flooring: Vec<Floor>,
    tilesheet: &'s Tilesheet,
    vao: VertexArray,
    pub background_color: Color,
}

#[derive(Debug, Error)]
pub enum MapLoadError {
    #[error("No player spawn: There must be a single player spawn object per level map.")]
    NoPlayerSpawn,
    #[error("No goals or crates: There must be at least one goal and one crate per level.")]
    NoGoalsOrCrates,
    #[error("Map not finite: The level's map must be set to finite in its properties.")]
    NotFinite,
    #[error(
        "Invalid layers: A level must at least two layers, one named \"building\" and \
    another named \"floor\"."
    )]
    InvalidLayers,
    #[error("Tilesheet load error: {0}")]
    TilesheetLoadError(
        #[from]
        #[source]
        TilesheetLoadError,
    ),
    #[error("Invalid object groups: The first and single object group in the map must be called 'objects'.")]
    InvalidObjectGroups,
    #[error("Invalid object: {0:?}")]
    InvalidObject(tiled::objects::Object),
    #[error("Tiled error: {0}")]
    TiledError(
        #[from]
        #[source]
        TiledError,
    ),
}

impl<'s> Level<'s> {
    pub fn from_map(data: &Map, assets: &'s mut AssetManager) -> Result<Level<'s>, MapLoadError> {
        if data.infinite {
            return Err(MapLoadError::NotFinite);
        }

        if data.tilesets.len() != 1 {
            todo!("Support for maps with multiple tilesets")
        }

        let tilesheet = {
            let tileset = data.tilesets[0].clone();
            let path = tileset.source.as_ref().unwrap().clone();
            assets.get_or_load_asset(&path, Tilesheet::from_tileset(tileset)?)
        };

        let size = Vector2u::new(data.width, data.height);

        let (building_layer, floor_layer) = match Self::get_building_and_floor_layers(&data) {
            Some(l) => l,
            None => return Err(MapLoadError::InvalidLayers),
        };

        let flooring = Self::extract_flooring(&building_layer, tilesheet.tileset());

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
                (object.y / data.tile_height as f32) as i32,
            );

            let object_tile = tilesheet.tileset().get_tile_by_gid(object.gid);

            match object_tile
                .and_then(|t| Some(t.tile_type.as_deref()))
                .flatten()
            {
                Some("spawn") => player_spawn = Some(position),
                Some("crate") => crates
                    .push(Crate::new(position, &tilesheet, object.gid).expect("crate creation")),
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
                _ => return Err(MapLoadError::InvalidObject(object.clone())),
            }
        }

        let player_spawn = match player_spawn {
            Some(spawn) => spawn,
            _ => return Err(MapLoadError::NoPlayerSpawn),
        };

        if goals.is_empty() || crates.is_empty() {
            return Err(MapLoadError::NoGoalsOrCrates);
        }

        let background_color = data
            .background_color
            .and_then(|c| Some(Color::rgb(c.red, c.green, c.blue)))
            .unwrap_or(Color::BLACK);

        Ok(Self {
            player_spawn,
            crates,
            goals,
            vao: Self::generate_vao(&size, &building_layer, &floor_layer, &tilesheet),
            flooring,
            size,
            tilesheet,
            background_color,
        })
    }

    pub fn from_file(path: &Path, assets: &'s mut AssetManager) -> Result<Self, MapLoadError> {
        let map = Map::parse_file(path)?;
        Self::from_map(&map, assets)
    }

    fn get_building_and_floor_layers(data: &Map) -> Option<(Vec<LayerTile>, Vec<LayerTile>)> {
        let building = data.layers.iter().filter(|l| l.name == "building").nth(0);
        let floor = data.layers.iter().filter(|l| l.name == "floor").nth(0);

        match (
            building.and_then(|x| Some(&x.tiles)),
            floor.and_then(|x| Some(&x.tiles)),
        ) {
            (Some(LayerData::Finite(building)), Some(LayerData::Finite(floor))) => Some((
                building.iter().flatten().copied().collect(),
                floor.iter().flatten().copied().collect(),
            )),
            _ => None,
        }
    }

    fn extract_flooring(building_layer: &Vec<LayerTile>, tileset: &Tileset) -> Vec<Floor> {
        building_layer
            .iter()
            .map(|tile| {
                if tile.gid == Gid::EMPTY {
                    return Floor::Walkable;
                }

                let tile_data = tileset.get_tile_by_gid(tile.gid);

                match tile_data.and_then(|t| t.tile_type.as_deref()) {
                    Some("solid") => Floor::Solid,
                    Some("hole") => Floor::Hole,
                    _ => Floor::Walkable,
                }
            })
            .collect::<Vec<_>>()
    }

    fn generate_vao(
        size_in_tiles: &Vector2u,
        building_layer: &Vec<LayerTile>,
        floor_layer: &Vec<LayerTile>,
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
            if f_tile.gid != Gid::EMPTY {
                quads.add_quad(
                    position + FLOOR_OFFSET,
                    1f32,
                    tilesheet
                        .tile_uv(f_tile.gid)
                        .expect("obtaining floor tile UV"),
                );
            }
            if b_tile.gid != Gid::EMPTY {
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
