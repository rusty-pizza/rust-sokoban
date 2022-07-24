use std::io::Cursor;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};

type Sound = Decoder<Cursor<Vec<u8>>>;

pub struct SoundManager {
    sounds_being_played: Vec<Sink>,
    output_stream: OutputStreamHandle,
}

impl SoundManager {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            sounds_being_played: Default::default(),
            output_stream: OutputStream::try_default()?.1,
        })
    }

    pub fn add_sound<'k>(&'k mut self, sound: Sound) {
        let mut sink = Sink::try_new(&self.output_stream).unwrap();
        sink.append(sound);

        self.sounds_being_played.push(sink);
    }

    pub fn update(&mut self) {
        self.sounds_being_played.retain(|sink| !sink.empty());
    }
}
