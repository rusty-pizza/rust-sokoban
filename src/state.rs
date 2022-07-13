use std::ops::ControlFlow;

use sfml::{
    graphics::{FloatRect, RenderWindow},
    window::Event,
};

use crate::context::Context;

pub struct LevelArray {
    pub rect: FloatRect,
    pub category: usize,
}

pub trait State<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s, '_, '_>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()>;

    fn process_event(
        &mut self,
        ctx: &mut Context<'s, '_, '_>,
        window: &mut RenderWindow,
        event: Event,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()>;
}

mod level_select;
pub use level_select::*;

mod playing;
pub use playing::*;

mod transitioning;
pub use transitioning::*;
