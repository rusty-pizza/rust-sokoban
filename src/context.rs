use std::{collections::HashSet, path::PathBuf, time::Duration};

use crate::{assets::AssetManager, sound_manager::SoundManager};

pub struct Context<'assets: 'sound, 'sound, 'other> {
    pub assets: &'assets AssetManager,
    pub sound: &'sound mut SoundManager<'assets>,
    pub completed_levels: &'other mut HashSet<PathBuf>,
    pub delta_time: Duration,
}
