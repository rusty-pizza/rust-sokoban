use sfml::graphics::Color;
use sfml::graphics::Rect;
use sfml::graphics::RenderTarget;
use sfml::graphics::RenderTexture;
use sfml::graphics::RenderWindow;
use sfml::graphics::Sprite;
use sfml::window::Event;

use std::ops::ControlFlow;
use std::time::Duration;

use crate::assets::AssetManager;
use crate::context::Context;

use super::State;

pub struct Transitioning<'s> {
    prev_state: Box<dyn State<'s> + 's>,
    // HACK: This is an option because `tick` does not move the state and as such we cannot move the next state out
    next_state: Option<Box<dyn State<'s> + 's>>,
    time_left: Duration,
}

impl<'s> Transitioning<'s> {
    const TRANSITION_TIME: Duration = Duration::from_millis(200);
    pub fn new(
        _assets: &'s AssetManager,
        prev_state: impl State<'s> + 's,
        next_state: impl State<'s> + 's,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            prev_state: Box::new(prev_state),
            next_state: Some(Box::new(next_state)),
            time_left: Self::TRANSITION_TIME,
        })
    }
}

impl<'s> State<'s> for Transitioning<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s>,
        _window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        // Update time left on transition
        self.time_left = self.time_left.saturating_sub(ctx.delta_time);

        if self.time_left.is_zero() {
            ControlFlow::Break(self.next_state.take().unwrap())
        } else {
            ControlFlow::Continue(())
        }
    }

    fn process_event(
        &mut self,
        _ctx: &mut Context<'s>,
        _window: &mut RenderWindow,
        _event: Event,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        ControlFlow::Continue(())
    }

    fn draw(&self, ctx: &mut Context<'s>, target: &mut dyn RenderTarget) {
        let mut render_target = RenderTexture::new(target.size().x, target.size().y).unwrap();

        self.next_state
            .as_ref()
            .unwrap()
            .draw(ctx, &mut render_target);

        let mut overlay_sprite = Sprite::with_texture_and_rect(
            render_target.texture(),
            &Rect {
                width: target.size().x as i32,
                height: -(target.size().y as i32),
                top: target.size().y as i32,
                ..Default::default()
            },
        );

        let transition_alpha = (255.
            - (self.time_left.as_secs_f32() / Self::TRANSITION_TIME.as_secs_f32()) * 255.)
            as u8;
        overlay_sprite.set_color(Color::rgba(255, 255, 255, transition_alpha));

        self.prev_state.draw(ctx, target);
        target.draw(&overlay_sprite);
    }
}
