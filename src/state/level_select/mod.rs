use std::ops::ControlFlow;

use sfml::{
    graphics::{BlendMode, FloatRect, Rect, RenderStates, RenderTarget},
    system::Vector2u,
    window::{Event, Key},
};

use sfml::graphics::Color;

use sfml::graphics::RenderWindow;

use crate::{
    context::{Context, SaveData},
    level::camera_transform,
    ui::{get_ui_obj_from_tiled_obj, update_button, ButtonState, UiObject},
};

use super::{playing::Playing, State, Transitioning};

mod ui;

use ui::*;

/// The level select screen. Uses the `main_menu` level in the asset manager to set up its layout.
#[derive(Clone)]
pub struct LevelSelect<'s> {
    drawables: Vec<Box<dyn UiObject<'s> + 's>>,
    level_arrays: Vec<LevelArray<'s>>,
}

impl<'s> LevelSelect<'s> {
    pub fn new(ctx: &Context<'s>) -> anyhow::Result<Self> {
        let mut drawables: Vec<Box<dyn UiObject + 's>> = Vec::new();
        let mut level_arrays = Vec::new();
        let assets = ctx.assets;

        let object_group = assets
            .main_menu
            .layers()
            .find_map(|l| l.as_object_layer())
            .unwrap();
        for object in object_group.objects() {
            if object.name == "level_array" {
                let (width, height) = match object.shape {
                    tiled::ObjectShape::Rect { width, height } => (width, height),
                    _ => panic!(),
                };
                let rect = FloatRect::new(object.x, object.y, width, height);
                let category = assets
                    .level_categories
                    .iter()
                    .enumerate()
                    .find(|(_, cat)| cat.name == object.user_type)
                    .expect("Unknown level category in level map")
                    .0;
                level_arrays.push(LevelArray::new(ctx, rect, category));
            } else if let Ok(obj) = get_ui_obj_from_tiled_obj(ctx, &assets.main_menu, &object) {
                drawables.push(obj);
            } else {
                log::warn!("could not parse object in level select: {:?}", object);
            }
        }

        Ok(Self {
            drawables,
            level_arrays,
        })
    }
}

impl<'s> State<'s> for LevelSelect<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        let mut level_to_transition_to = None;
        for level_array in self.level_arrays.iter_mut() {
            let category = &ctx.assets.level_categories[level_array.category];

            for level_idx in 0..category.maps.len() {
                let level_button = &mut level_array.sprites[level_idx];
                if level_button.unlocked() {
                    if update_button(ctx, window, &mut level_button.sprite) == ButtonState::Pressed
                    {
                        // Lifetime shenanigans: Can't return here because we need access to self, which is currently being mutably borrowed
                        level_to_transition_to = Some((level_idx, level_array.category));
                        break;
                    }
                }
            }
        }

        if let Some((idx, category)) = level_to_transition_to {
            ControlFlow::Break(Box::new(
                Transitioning::new(
                    ctx.assets,
                    self.clone(),
                    Playing::new(ctx, idx, category).unwrap(),
                )
                .unwrap(),
            ))
        } else {
            ControlFlow::Continue(())
        }
    }

    fn process_event(
        &mut self,
        ctx: &mut Context<'s>,
        window: &mut RenderWindow,
        event: Event,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        match event {
            Event::Resized { width, height } => {
                let view = sfml::graphics::View::from_rect(Rect {
                    left: 0.,
                    top: 0.,
                    width: width as f32,
                    height: height as f32,
                });
                window.set_view(&view);

                *self = LevelSelect::new(ctx).unwrap();
            }

            #[cfg(debug_assertions)]
            // Unlock all levels when Ctrl+I is pressed
            Event::KeyPressed {
                code: Key::I,
                ctrl: true,
                ..
            } => {
                for category in ctx.assets.level_categories.iter() {
                    for level in category.maps.iter() {
                        ctx.completed_levels.complete_lvl(level.1.clone());
                    }
                }

                *self = LevelSelect::new(ctx).unwrap();
            }

            // Reset progress when Ctrl+N is pressed
            Event::KeyPressed {
                code: Key::N,
                ctrl: true,
                ..
            } => {
                ctx.completed_levels = SaveData::default();

                *self = LevelSelect::new(ctx).unwrap();
            }

            _ => (),
        }

        ControlFlow::Continue(())
    }

    fn draw(&self, ctx: &mut Context<'s>, target: &mut dyn RenderTarget) {
        let camera_transform = camera_transform(
            target.size(),
            Vector2u::new(
                ctx.assets.main_menu.width * ctx.assets.main_menu.tile_width,
                ctx.assets.main_menu.height * ctx.assets.main_menu.tile_height,
            ),
            0.,
        );
        let render_states = RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

        target.clear(
            ctx.assets
                .main_menu
                .background_color
                .map_or(Color::BLACK, |c| Color::rgb(c.red, c.green, c.blue)),
        );

        for drawable in self.drawables.iter() {
            target.draw_with_renderstates(drawable.as_drawable(), &render_states);
        }

        for level_array in self.level_arrays.iter() {
            for button in level_array.sprites.iter() {
                target.draw_with_renderstates(&button.sprite, &render_states);
                if let Some(lock) = button.lock_sprite.as_ref() {
                    target.draw_with_renderstates(lock, &render_states);
                }
            }
        }
    }
}
