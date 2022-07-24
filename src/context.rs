mod level_completion_db;
pub use level_completion_db::*;

use std::time::Duration;

use crate::{assets::AssetManager, sound_manager::SoundManager};

pub struct Context<'assets> {
    pub assets: &'assets AssetManager,
    pub sound: SoundManager<'assets>,
    pub completed_levels: LevelCompletionDb,
    pub delta_time: Duration,
}
