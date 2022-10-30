use sfml::graphics::Rect;

use sfml;

use sfml::graphics::RenderTarget;

use sfml::graphics::Sprite;
use sfml::graphics::Transformable;

use sfml::system::Vector2u;
use sfml::window::Key;

use super::transitioning::Transitioning;
use super::LevelSelect;

use std;

use sfml::window::Event;

use sfml::system::Vector2f;

use sfml::graphics::Text;

use sfml::graphics::BlendMode;

use sfml::graphics::RenderStates;

#[cfg(feature = "editor")]
use guiedit::RenderWindow;
#[cfg(not(feature = "editor"))]
use sfml::graphics::RenderWindow;

use std::ops::ControlFlow;

use crate::context::Context;
use crate::level::camera_transform;
use crate::ui::get_ui_obj_from_tiled_obj;
use crate::ui::sprite_from_tiled_obj;
use crate::ui::update_button;
use crate::ui::ButtonState;
use crate::ui::UiObject;

use super::State;

use crate::level::Level;

#[derive(Clone)]
pub struct PlayOverlay<'s> {
    overlay: Vec<Box<dyn UiObject<'s> + 's>>,
    back_button: Sprite<'s>,
}

#[derive(Clone)]
pub struct Playing<'s> {
    level_index: usize,
    category_index: usize,
    level: Level<'s>,
    overlay: PlayOverlay<'s>,
}

impl<'s> Playing<'s> {
    pub fn new(
        ctx: &Context<'s>,
        level_index: usize,
        category_index: usize,
    ) -> anyhow::Result<Self> {
        let mut overlay = Vec::new();
        let mut back_button = None;
        for object in ctx.assets.play_overlay_map.object_groups[0].objects.iter() {
            if object.name == "back_button" {
                let sprite =
                    sprite_from_tiled_obj(ctx.assets, &ctx.assets.play_overlay_map, object)?;
                back_button = Some(sprite);
            } else if let Ok(obj) =
                get_ui_obj_from_tiled_obj(ctx, &ctx.assets.play_overlay_map, object)
            {
                overlay.push(obj);
            } else {
                log::warn!("could not parse object in play overlay: {:?}", object);
            }
        }

        Ok(Self {
            level_index,
            category_index,
            level: Level::from_map(
                &ctx.assets.level_categories[category_index].maps[level_index],
                ctx,
            )?,
            overlay: PlayOverlay {
                overlay,
                back_button: back_button.expect("found no back button in play overlay"),
            },
        })
    }
}

impl<'s> State<'s> for Playing<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        self.level.update(ctx, ctx.delta_time);

        match update_button(ctx, window, &mut self.overlay.back_button) {
            ButtonState::Pressed => {
                let next_state =
                    Transitioning::new(ctx.assets, self.clone(), LevelSelect::new(ctx).unwrap())
                        .unwrap();

                ControlFlow::Break(Box::new(next_state))
            }
            _ => ControlFlow::Continue(()),
        }
    }

    fn process_event(
        &mut self,
        ctx: &mut Context<'s>,
        window: &mut RenderWindow,
        event: Event,
    ) -> ControlFlow<Box<(dyn State<'s> + 's)>> {
        let is_level_won = self.level.is_won();

        match event {
            Event::KeyPressed { .. } if is_level_won => {
                // Mark this level as complete
                ctx.completed_levels.complete_lvl(
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
                    return ControlFlow::Break(Box::new(
                        Transitioning::new(
                            ctx.assets,
                            self.clone(),
                            LevelSelect::new(ctx).unwrap(),
                        )
                        .unwrap(),
                    ));
                } else {
                    // Go to next level
                    return ControlFlow::Break(Box::new(
                        Transitioning::new(
                            ctx.assets,
                            self.clone(),
                            Playing::new(ctx, next_level_index, self.category_index).unwrap(),
                        )
                        .unwrap(),
                    ));
                }
            }
            Event::KeyPressed {
                code: Key::Escape, ..
            } => {
                return ControlFlow::Break(Box::new(
                    Transitioning::new(ctx.assets, self.clone(), LevelSelect::new(ctx).unwrap())
                        .unwrap(),
                ));
            }
            Event::KeyPressed { code: Key::R, .. } => {
                self.level = Level::from_map(
                    &ctx.assets.level_categories[self.category_index].maps[self.level_index],
                    ctx,
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

    fn draw(&self, ctx: &mut Context<'s>, target: &mut dyn RenderTarget) {
        let is_level_won = self.level.is_won();

        let transform = camera_transform(
            target.size(),
            Vector2u::new(
                // HACK: This should refer to the level tile_width/height, but it refers to the tilesheet tilesize, which might not always coincide
                self.level.tilemap().size().x * self.level.tilesheet().tile_size().x,
                self.level.tilemap().size().y * self.level.tilesheet().tile_size().y,
            ),
            self.level.tilesheet().tile_size().y as f32 * 2.,
        );
        let render_states = RenderStates::new(BlendMode::ALPHA, transform, None, None);

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

            let mut moves_text = Text::new(
                format!("Used {} moves", self.level.action_count()).as_str(),
                &ctx.assets.win_font,
                30,
            );
            moves_text.set_position(Vector2f::new(
                target.size().x as f32 / 2. - moves_text.global_bounds().width / 2.,
                text.position().y + text.global_bounds().height + 20.,
            ));
            target.draw_with_renderstates(&moves_text, &RenderStates::DEFAULT);

            let mut subtext = Text::new("Press any key to continue", &ctx.assets.win_font, 30);
            subtext.set_position(Vector2f::new(
                target.size().x as f32 / 2. - subtext.global_bounds().width / 2.,
                moves_text.position().y + moves_text.global_bounds().height + 20.,
            ));
            target.draw_with_renderstates(&subtext, &RenderStates::DEFAULT);
        }

        let transform = camera_transform(
            target.size(),
            Vector2u::new(
                ctx.assets.play_overlay_map.width * ctx.assets.play_overlay_map.tile_width,
                ctx.assets.play_overlay_map.height * ctx.assets.play_overlay_map.tile_height,
            ),
            0.,
        );
        let render_states = RenderStates::new(BlendMode::ALPHA, transform, None, None);

        for obj in self.overlay.overlay.iter() {
            target.draw_with_renderstates(obj.as_drawable(), &render_states);
        }
        target.draw_with_renderstates(&self.overlay.back_button, &render_states);
    }
}
