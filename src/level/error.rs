use thiserror::Error;
use tiled::error::TiledError;

use crate::graphics::TilesheetLoadError;

#[derive(Debug, Error)]
pub enum LevelLoadError {
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
