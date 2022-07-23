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

use crate::assets::AssetManager;
use crate::context::Context;
use crate::level::camera_transform;

use super::State;

use crate::level::Level;

#[derive(Clone)]
pub struct Playing<'s> {
    level_index: usize,
    category_index: usize,
    pub(crate) level: Level<'s>,
}

impl<'s> Playing<'s> {
    pub fn new(
        assets: &'s AssetManager,
        level_index: usize,
        category_index: usize,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            level_index,
            category_index,
            level: Level::from_map(
                &assets.level_categories[category_index].maps[level_index],
                assets,
            )?,
        })
    }
}

impl<'s> State<'s> for Playing<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s, '_, '_>,
        _window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        self.level.update(ctx, ctx.delta_time);

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
                    ctx.assets.level_categories[self.category_index].maps[self.level_index]
                        .source
                        .clone()
                        .unwrap(),
                );

                let next_level_index = self.level_index + 1;

                if self.level_index + 1
                    >= ctx.assets.level_categories[self.category_index].maps.len()
                {
                    // Go back to level select if category or game is finished
                    return ControlFlow::Break(Box::new(LevelSelect::new(ctx)));
                } else {
                    // Go to next level
                    return ControlFlow::Break(Box::new(
                        Transitioning::new(
                            ctx.assets,
                            self.clone(),
                            Playing::new(ctx.assets, next_level_index, self.category_index)
                                .unwrap(),
                        )
                        .unwrap(),
                    ));
                }
            }
            Event::KeyPressed {
                code: Key::Escape, ..
            } => {
                return ControlFlow::Break(Box::new(LevelSelect::new(ctx)));
            }
            Event::KeyPressed { code: Key::R, .. } => {
                self.level = Level::from_map(
                    &ctx.assets.level_categories[self.category_index].maps[self.level_index],
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

    fn draw(&self, ctx: &mut Context<'s, '_, '_>, target: &mut dyn RenderTarget) {
        let is_level_won = self.level.is_won();

        let camera_transform = camera_transform(target.size(), self.level.tilemap().size());
        let render_states = RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

        target.clear(self.level.background_color);

        target.draw_with_renderstates(&self.level, &render_states);

        if is_level_won {
            let is_last_level_of_category =
                self.level_index + 1 >= ctx.assets.level_categories[self.category_index].maps.len();
            let text = if is_last_level_of_category {
                "Category complete!"
            } else {
                "Level complete!"
            };
            let mut text = Text::new(text, &ctx.assets.win_font, 60);
            text.set_position(Vector2f::new(
                target.size().x as f32 / 2. - text.global_bounds().width / 2.,
                10.,
            ));
            target.draw_with_renderstates(&text, &RenderStates::DEFAULT);
            let mut subtext = Text::new("Press any key to continue", &ctx.assets.win_font, 30);
            subtext.set_position(Vector2f::new(
                target.size().x as f32 / 2. - subtext.global_bounds().width / 2.,
                10. + text.global_bounds().height + 20.,
            ));
            target.draw_with_renderstates(&subtext, &RenderStates::DEFAULT);
        }
    }
}
