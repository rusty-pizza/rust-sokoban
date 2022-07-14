use std::{collections::HashSet, path::PathBuf, time::Duration};

use crate::{assets::AssetManager, sound_manager::SoundManager};

pub struct Context<'assets: 'sound, 'sound, 'other> {
    pub assets: &'assets AssetManager,
    pub sound: &'sound mut SoundManager<'assets>,
    pub completed_levels: &'other mut HashSet<PathBuf>,
    // TODO: Remove this, just switch to next level on the playing state
    pub current_category_idx: &'other mut usize,
    pub current_level_idx: &'other mut usize,
    pub delta_time: Duration,
}
