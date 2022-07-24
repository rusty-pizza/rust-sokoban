mod level_completion_db;
pub use level_completion_db::*;

use std::time::Duration;

use crate::{assets::AssetManager, sound_manager::SoundManager};

pub struct Context<'assets: 'sound, 'sound, 'other> {
    pub assets: &'assets AssetManager,
    pub sound: &'sound mut SoundManager<'assets>,
    pub completed_levels: &'other mut LevelCompletionDb,
    pub delta_time: Duration,
}
