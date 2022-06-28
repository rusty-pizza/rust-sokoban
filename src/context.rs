use crate::{assets::AssetManager, sound_manager::SoundManager};

pub struct Context<'c, 's: 'c> {
    pub assets: &'s AssetManager,
    pub sound: &'c mut SoundManager<'s>,
}
