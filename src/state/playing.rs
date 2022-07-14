use sfml::graphics::Rect;

use sfml;

use sfml::graphics::RenderTarget;

use sfml::graphics::Transformable;

use sfml::window::Key;

use super::transitioning::Transitioning;
use super::LevelSelect;

use std;

use sfml::window::Event;

use sfml::system::Vector2f;

use sfml::graphics::Text;

use sfml::graphics::BlendMode;

use sfml::graphics::RenderStates;

use std::ops::ControlFlow;

use sfml::graphics::RenderWindow;

use crate::context::Context;
use crate::level::camera_transform;

use super::State;

use crate::level::Level;

pub struct Playing<'s> {
    pub(crate) level: Level<'s>,
}

impl<'s> State<'s> for Playing<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s, '_, '_>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        let is_level_won = self.level.is_won();

        // Update
        self.level.update(ctx, ctx.delta_time);

        // Render frame
        let camera_transform = camera_transform(window.size(), self.level.tilemap().size());
        let render_states = RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

        window.clear(self.level.background_color);

        window.draw_with_renderstates(&self.level, &render_states);

        if is_level_won {
            let mut text = Text::new("Level complete!", &ctx.assets.win_font, 60);
            text.set_position(Vector2f::new(
                window.size().x as f32 / 2. - text.global_bounds().width / 2.,
                10.,
            ));
            window.draw_with_renderstates(&text, &RenderStates::DEFAULT);
            let mut subtext = Text::new("Press any key to continue", &ctx.assets.win_font, 30);
            subtext.set_position(Vector2f::new(
                window.size().x as f32 / 2. - subtext.global_bounds().width / 2.,
                10. + text.global_bounds().height + 20.,
            ));
            window.draw_with_renderstates(&subtext, &RenderStates::DEFAULT);
        }

        ControlFlow::Continue(())
    }

    fn process_event(
        &mut self,
        ctx: &mut Context<'s, '_, '_>,
        window: &mut RenderWindow,
        event: Event,
    ) -> ControlFlow<Box<(dyn State<'s> + 's)>> {
        let is_level_won = self.level.is_won();

        match event {
            Event::KeyPressed { .. } if is_level_won => {
                // Mark this level as complete
                ctx.completed_levels.insert(
                    ctx.assets.level_categories[*ctx.current_category_idx].maps
                        [*ctx.current_level_idx]
                        .source
                        .clone()
                        .unwrap(),
                );

                // Go to next level
                *ctx.current_level_idx += 1;

                if *ctx.current_level_idx
                    >= ctx.assets.level_categories[*ctx.current_category_idx]
                        .maps
                        .len()
                {
                    *ctx.current_level_idx = 0;
                    *ctx.current_category_idx += 1;
                }

                if *ctx.current_category_idx >= ctx.assets.level_categories.len() {
                    println!("You won!");
                    std::process::exit(0);
                } else {
                    return ControlFlow::Break(Box::new(Transitioning::new(
                        self.level.clone(),
                        Level::from_map(
                            &ctx.assets.level_categories[*ctx.current_category_idx].maps
                                [*ctx.current_level_idx],
                            &ctx.assets,
                        )
                        .unwrap(),
                    )));
                }
            }
            Event::KeyPressed {
                code: Key::Escape, ..
            } => {
                return ControlFlow::Break(Box::new(LevelSelect::new(
                    ctx.assets,
                    window,
                    ctx.completed_levels.len(),
                )));
            }
            Event::KeyPressed { code: Key::R, .. } => {
                self.level = Level::from_map(
                    &ctx.assets.level_categories[*ctx.current_category_idx].maps
                        [*ctx.current_level_idx],
                    &ctx.assets,
                )
                .unwrap()
            }
            Event::Resized { width, height } => {
                let view = sfml::graphics::View::from_rect(&Rect {
                    left: 0.,
                    top: 0.,
                    width: width as f32,
                    height: height as f32,
                });
                window.set_view(&view);
            }
            _ => self.level.handle_event(ctx, event),
        }

        ControlFlow::Continue(())
    }
}
