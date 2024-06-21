use std::ops::ControlFlow;

use ggez::graphics::Canvas;
use sfml::{graphics::RenderWindow, window::Event};

use crate::context::Context;

pub trait State<'s> {
    // Sadly this function cannot move `self` because that would make it object unsafe
    fn tick(
        &mut self,
        ctx: &mut Context<'s>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()>;

    fn draw(&self, ctx: &mut Context<'s>, target: &mut Canvas);

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
