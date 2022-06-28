use sfml::audio::{Sound, SoundStatus};

pub struct SoundManager<'s> {
    sounds_being_played: Vec<Sound<'s>>,
}

impl<'s> SoundManager<'s> {
    pub fn new() -> Self {
        Self {
            sounds_being_played: Default::default(),
        }
    }

    pub fn add_sound<'k>(&'k mut self, sound: Sound<'s>) {
        self.sounds_being_played.push(sound);
    }

    pub fn update(&mut self) {
        self.sounds_being_played
            .retain(|sound| sound.status() == SoundStatus::PLAYING);
    }
}
