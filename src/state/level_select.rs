use std::ops::ControlFlow;

use sfml::{
    graphics::{BlendMode, FloatRect, Rect, RenderStates, RenderTarget, Sprite, Transformable},
    system::Vector2u,
    window::{Event, Key},
};

use tiled::{self, tile::Gid};

#[cfg(feature = "editor")]
use guiedit::RenderWindow;
#[cfg(not(feature = "editor"))]
use sfml::graphics::RenderWindow;

use sfml::graphics::Color;

use crate::{
    context::{Context, SaveData},
    level::camera_transform,
    ui::{get_ui_obj_from_tiled_obj, update_button, ButtonState, UiObject},
};

use sfml::system::Vector2f;

use super::{playing::Playing, State, Transitioning};

#[derive(Clone)]
struct LevelArrayButton<'s> {
    pub sprite: Sprite<'s>,
    pub lock_sprite: Option<Sprite<'s>>,
}

impl LevelArrayButton<'_> {
    pub fn unlocked(&self) -> bool {
        self.lock_sprite.is_none()
    }
}

#[derive(Clone)]
struct LevelArray<'s> {
    pub category: usize,
    pub sprites: Vec<LevelArrayButton<'s>>,
}

impl<'s> LevelArray<'s> {
    fn new(ctx: &Context<'s>, rect: FloatRect, category_idx: usize) -> Self {
        let mut buttons = Vec::new();

        let mut level_icon = ctx.assets.icon_tilesheet.tile_sprite(Gid(92)).unwrap();
        let mut lock_icon = ctx.assets.icon_tilesheet.tile_sprite(Gid(116)).unwrap();
        let category = &ctx.assets.level_categories[category_idx];
        level_icon.set_position(Vector2f::new(rect.left, rect.top));
        level_icon.set_scale(Vector2f::new(
            rect.height / level_icon.global_bounds().height,
            rect.height / level_icon.global_bounds().height,
        ));
        lock_icon.set_position(Vector2f::new(rect.left, rect.top));
        lock_icon.set_scale(Vector2f::new(
            rect.height / lock_icon.global_bounds().height,
            rect.height / lock_icon.global_bounds().height,
        ));

        let mut completed_previous_level = true;
        for level in category.maps.iter() {
            let completed_level = ctx
                .completed_levels
                .internal_set()
                .contains(level.source.as_ref().unwrap());
            let mut color = category.color;
            let mut draw_lock = false;
            if !(completed_level || completed_previous_level) {
                color = category.color;
                color.a = 50;
                draw_lock = true;
            }
            level_icon.set_color(color);
            buttons.push(LevelArrayButton {
                sprite: level_icon.clone(),
                lock_sprite: draw_lock.then_some(lock_icon.clone()),
            });

            level_icon.move_(Vector2f::new(level_icon.global_bounds().width, 0.));
            lock_icon.move_(Vector2f::new(level_icon.global_bounds().width, 0.));

            completed_previous_level = completed_level;
        }

        Self {
            sprites: buttons,
            category: category_idx,
        }
    }
}

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

        for object in assets.main_menu.object_groups[0].objects.iter() {
            if object.name == "level_array" {
                let rect = FloatRect::new(object.x, object.y, object.width, object.height);
                let category = assets
                    .level_categories
                    .iter()
                    .enumerate()
                    .find(|(_, cat)| cat.name == object.obj_type)
                    .expect("Unknown level category in level map")
                    .0;
                level_arrays.push(LevelArray::new(ctx, rect, category));
            } else if let Ok(obj) = get_ui_obj_from_tiled_obj(ctx, &assets.main_menu, object) {
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
                        level_to_transition_to = Some((level_idx, level_array.category));
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
                let view = sfml::graphics::View::from_rect(&Rect {
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
                        ctx.completed_levels
                            .complete_lvl(level.source.clone().unwrap());
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
