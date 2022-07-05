use std::time::Duration;

use crate::level::Level;

pub enum PlayState<'s> {
    Playing {
        level: Level<'s>,
    },
    Transitioning {
        prev_level: Level<'s>,
        next_level: Level<'s>,
        time_left: Duration,
    },
}
