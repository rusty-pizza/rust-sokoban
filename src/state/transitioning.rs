use sfml::graphics::RenderTarget;
use sfml::graphics::Shape;
use sfml::graphics::Transformable;
use sfml::window::Event;

use sfml::system::Vector2f;

use sfml::graphics::RectangleShape;

use sfml::graphics::BlendMode;

use sfml::graphics::RenderStates;

use std::ops::ControlFlow;

use sfml::graphics::RenderWindow;

use crate::context::Context;
use crate::level::camera_transform;

use super::Playing;
use super::State;

use std::time::Duration;

use crate::level::Level;

pub struct Transitioning<'s> {
    pub(crate) prev_level: Level<'s>,
    pub(crate) next_level: Level<'s>,
    pub(crate) time_left: Duration,
}

impl<'s> Transitioning<'s> {
    pub(crate) const TRANSITION_TIME: Duration = Duration::from_secs(1);
    pub(crate) fn new(prev_level: Level<'s>, next_level: Level<'s>) -> Self {
        Self {
            prev_level,
            next_level,
            time_left: Self::TRANSITION_TIME,
        }
    }
}

impl<'s> State<'s> for Transitioning<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s, '_, '_>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        let is_fading_out = self.time_left > Self::TRANSITION_TIME / 2;

        let mut transition_color = self.prev_level.background_color;
        *transition_color.alpha_mut() = (255.
            - ((self.time_left.as_secs_f32() / (Self::TRANSITION_TIME.as_secs_f32() / 2.)) - 1.)
                .abs()
                * 255.) as u8;
        let current_level = if is_fading_out {
            &self.prev_level
        } else {
            &self.next_level
        };

        // Render frame
        let camera_transform = camera_transform(window.size(), current_level.tilemap().size());
        let render_states = RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

        // TODO: Cache shape
        let mut transition_overlay = RectangleShape::with_size(Vector2f::new(
            current_level.tilemap().size().x as f32 + 10.,
            current_level.tilemap().size().y as f32 + 10.,
        ));
        transition_overlay.set_position(Vector2f::new(-5., -5.));

        // TODO: Transition between both background colors
        transition_overlay.set_fill_color(transition_color);

        window.clear(current_level.background_color);

        window.draw_with_renderstates(current_level, &render_states);
        window.draw_with_renderstates(&transition_overlay, &render_states);

        // Update time left on transition
        self.time_left = self.time_left.saturating_sub(ctx.delta_time);

        if self.time_left.is_zero() {
            ControlFlow::Break(Box::new(Playing {
                level: current_level.clone(),
            }))
        } else {
            ControlFlow::Continue(())
        }
    }

    fn process_event(
        &mut self,
        ctx: &mut Context<'s, '_, '_>,
        window: &mut RenderWindow,
        event: Event,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        ControlFlow::Continue(())
    }
}
