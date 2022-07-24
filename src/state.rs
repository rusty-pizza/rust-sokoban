use std::ops::ControlFlow;

use sfml::{
    graphics::{FloatRect, RenderTarget, RenderWindow},
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
        ctx: &mut Context<'s>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()>;

    fn draw(&self, ctx: &mut Context<'s>, target: &mut dyn RenderTarget);

    fn process_event(
        &mut self,
        ctx: &mut Context<'s>,
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
