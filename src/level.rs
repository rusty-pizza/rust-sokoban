use std::{num::NonZeroU32, path::Path};

use sfml::{
    graphics::{Color, Drawable, Sprite, Transformable, VertexArray},
    system::{Vector2f, Vector2i, Vector2u},
};
use tiled::{
    error::TiledError,
    layers::{LayerData, LayerTile},
    map::Map,
    properties::PropertyValue,
    tile::Gid,
};

use crate::{
    asset_manager::AssetManager,
    quadarray::QuadArray,
    sprite_atlas::SpriteAtlas,
    tilesheet::{Tilesheet, TilesheetLoadError},
};

use thiserror::Error;

// TODO: Integrate this into the Tiled crate
fn get_tile_by_gid(tilesheet: &Tilesheet, gid: Gid) -> Option<&tiled::tile::Tile> {
    let id = gid.0 - tilesheet.tileset().first_gid.0;
    // FIXME: This won't return tiles with no special characteristics (tiled crate only keeps track
    // of special ones)
    match tilesheet
        .tileset()
        .tiles
        .binary_search_by_key(&id, |t| t.id)
    {
        Ok(index) => Some(&tilesheet.tileset().tiles[index]),
        _ => None,
    }
}

enum CrateType {
    WithId(NonZeroU32),
    Any,
}

impl Default for CrateType {
    fn default() -> Self {
        CrateType::Any
    }
}

impl CrateType {
    fn from_tiled_property(prop: &PropertyValue) -> Self {
        match prop {
            PropertyValue::IntValue(style) => match NonZeroU32::new(*style as u32) {
                Some(x) => CrateType::WithId(x),
                None => CrateType::Any,
            },
            _ => CrateType::Any,
        }
    }
}

pub struct Crate<'s> {
    position: Vector2i,
    sprite_atlas: SpriteAtlas<'s>,
    crate_type: CrateType,
}

impl<'s> Crate<'s> {
    const NORMAL_FRAME: usize = 0;
    const DROPPED_FRAME: usize = 1;
    const POSITIONED_FRAME: usize = 2;

    fn new(position: Vector2i, tilesheet: &'s Tilesheet, gid: Gid) -> Option<Self> {
        let tile = get_tile_by_gid(tilesheet, gid)?;

        let crate_type = tile
            .properties
            .0
            .iter()
            .find(|&(name, _)| name == "style")
            .and_then(|(_, prop)| Some(CrateType::from_tiled_property(prop)))
            .unwrap_or_default();

        let normal_tex_rect = tilesheet.tile_rect(gid)?;
        let dropped_tex_rect = tilesheet.tile_rect(Gid(tile
            .animation
            .as_ref()?
            .frames
            .get(Self::DROPPED_FRAME)?
            .tile_id
            + tilesheet.tileset().first_gid.0))?;
        let positioned_tex_rect = tilesheet.tile_rect(Gid(tile
            .animation
            .as_ref()?
            .frames
            .get(Self::POSITIONED_FRAME)?
            .tile_id
            + tilesheet.tileset().first_gid.0))?;

        let sprite_atlas = {
            let mut sprite_atlas = SpriteAtlas::with_texture_and_frames(
                tilesheet.texture(),
                &[normal_tex_rect, dropped_tex_rect, positioned_tex_rect],
            );
            sprite_atlas.set_position(Vector2f::new(position.x as f32, position.y as f32));
            sprite_atlas.set_scale(Vector2f::new(
                1f32 / tilesheet.tile_size().x as f32,
                1f32 / tilesheet.tile_size().y as f32,
            ));
            sprite_atlas
        };

        Some(Self {
            position,
            crate_type,
            sprite_atlas,
        })
    }
}

impl<'s> Drawable for Crate<'s> {
    fn draw<'a: 'shader, 'texture, 'shader, 'shader_texture>(
        &'a self,
        target: &mut dyn sfml::graphics::RenderTarget,
        states: &sfml::graphics::RenderStates<'texture, 'shader, 'shader_texture>,
    ) {
        self.sprite_atlas.draw(target, states);
    }
}

pub struct Goal<'s> {
    position: Vector2i,
    accepted_type: CrateType,
    sprite: Sprite<'s>,
}

impl<'s> Goal<'s> {
    fn new(
        position: Vector2i,
        accepted_style: CrateType,
        tilesheet: &'s Tilesheet,
        gid: Gid,
    ) -> Option<Self> {
        tilesheet.tile_sprite(gid).map(|mut sprite| {
            sprite.set_position(Vector2f::new(position.x as f32, position.y as f32));
            sprite.set_scale({
                let rect = sprite.texture_rect();
                Vector2f::new(1f32 / rect.width as f32, 1f32 / rect.height as f32)
            });
            Self {
                position,
                accepted_type: accepted_style,
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

enum Tile {
    Solid,
    Hole,
}

pub struct Level<'s> {
    player_spawn: Vector2i,
    crates: Vec<Crate<'s>>,
    goals: Vec<Goal<'s>>,
    size: Vector2u,
    tiles: Vec<Option<Tile>>,
    tilesheet: &'s Tilesheet,
    vao: VertexArray,
    background_color: Color,
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
        "Invalid layers: A level must contain exactly two layers, one named \"building\" and \
    another named \"floor\"."
    )]
    InvalidLayers,
    #[error("Tilesheet load error: {0}")]
    TilesheetLoadError(
        #[from]
        #[source]
        TilesheetLoadError,
    ),
    #[error("Invalid object groups")]
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

        if data.layers.len() > 2 {
            return Err(MapLoadError::InvalidLayers);
        }

        if data.tilesets.len() != 1 {
            // TODO: Support for maps with multiple tilesets
            todo!()
        }

        let size = Vector2u::new(data.width, data.height);

        let (building_layer, floor_layer): (Vec<LayerTile>, Vec<LayerTile>) = {
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

        let tilesheet = {
            let tileset = data.tilesets[0].clone();
            let path = tileset.source.as_ref().unwrap().clone();
            assets.get_or_load_asset(&path, Tilesheet::from_tileset(tileset)?)
        };

        let tiles = building_layer
            .iter()
            .map(|tile| {
                if tile.gid == Gid::EMPTY {
                    return None;
                }

                let tile_data = get_tile_by_gid(tilesheet, tile.gid);

                match tile_data.and_then(|t| t.tile_type.as_deref()) {
                    Some("solid") => Some(Tile::Solid),
                    Some("hole") => Some(Tile::Hole),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

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

            let object_tile = get_tile_by_gid(&tilesheet, object.gid);

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

        let background_color = {
            let c = data.background_color.unwrap_or(tiled::properties::Color {
                red: 0,
                green: 0,
                blue: 0,
            });
            Color::rgb(c.red, c.green, c.blue)
        };

        Ok(Self {
            player_spawn,
            crates,
            goals,
            vao: Self::generate_vao(&size, &building_layer, &floor_layer, &tilesheet),
            tiles,
            size,
            tilesheet,
            background_color,
        })
    }

    pub fn from_file(path: &Path, assets: &'s mut AssetManager) -> Result<Self, MapLoadError> {
        let map = Map::parse_file(path)?;
        Self::from_map(&map, assets)
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
