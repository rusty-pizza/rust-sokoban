use std::time::Duration;

use sfml::{
    graphics::{Drawable, FloatRect, Rect, Sprite, Text},
    system::Vector2f,
    SfBox,
};

use crate::level::Level;

pub struct LevelArray {
    pub rect: FloatRect,
    pub category: usize,
}

pub enum PlayState<'s> {
    LevelSelect {
        texts: Vec<Text<'s>>,
        level_arrays: Vec<LevelArray>,
        viewport_offset: Vector2f,
    },
    Playing {
        level: Level<'s>,
    },
    Transitioning {
        prev_level: Level<'s>,
        next_level: Level<'s>,
        time_left: Duration,
    },
}
