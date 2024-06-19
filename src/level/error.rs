use thiserror::Error;

use crate::graphics::TilesheetLoadError;

/// Represents an error that has occurred while loading a level.
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
    #[error("Invalid object groups: There should be a single and only object group in the map.")]
    InvalidObjectGroups,
    #[error("Invalid object: {0:?}")]
    InvalidObject(tiled::ObjectData),
    #[error("Tiled error: {0}")]
    TiledError(
        #[from]
        #[source]
        tiled::Error,
    ),
}
