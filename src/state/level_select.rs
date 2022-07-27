use std::ops::ControlFlow;

use sfml::{
    audio::{Sound, SoundSource},
    graphics::{BlendMode, FloatRect, Rect, RenderStates, RenderTarget, Transformable},
    system::Vector2u,
    window::{Event, Key},
};

use tiled::{self, tile::Gid};

use sfml::graphics::Color;

use sfml::graphics::RenderWindow;

use crate::{
    context::Context,
    level::camera_transform,
    ui::{get_ui_obj_from_tiled_obj, UiObject},
};

use sfml::system::Vector2f;

use super::{playing::Playing, State, Transitioning};

#[derive(Clone, Copy)]
pub struct LevelArray {
    pub rect: FloatRect,
    pub category: usize,
}

#[derive(Clone)]
pub struct LevelSelect<'s> {
    drawables: Vec<Box<dyn UiObject<'s> + 's>>,
    level_arrays: Vec<LevelArray>,
    clicked: bool,
    level_hovered: Option<(usize, usize)>,
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
                level_arrays.push(LevelArray { rect, category });
            } else if let Ok(obj) = get_ui_obj_from_tiled_obj(ctx, &assets.main_menu, object) {
                drawables.push(obj);
            } else {
                log::warn!("could not parse object in level select: {:?}", object);
            }
        }

        Ok(Self {
            drawables,
            level_arrays,
            clicked: false,
            level_hovered: None,
        })
    }
}

impl<'s> State<'s> for LevelSelect<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        let mut next_state: Option<Box<dyn State<'s> + 's>> = None;
        let camera_transform = camera_transform(
            window.size(),
            Vector2u::new(
                ctx.assets.main_menu.width * ctx.assets.main_menu.tile_width,
                ctx.assets.main_menu.height * ctx.assets.main_menu.tile_height,
            ),
            0.,
        );

        for level_array in self.level_arrays.iter() {
            let mut level_icon = ctx.assets.icon_tilesheet.tile_sprite(Gid(92)).unwrap();
            let category = &ctx.assets.level_categories[level_array.category];
            level_icon.set_position(Vector2f::new(level_array.rect.left, level_array.rect.top));
            level_icon.set_scale(Vector2f::new(
                level_array.rect.height / level_icon.global_bounds().height,
                level_array.rect.height / level_icon.global_bounds().height,
            ));

            let mut completed_previous_level = true;
            for (level_idx, level) in category.maps.iter().enumerate() {
                let completed_level = ctx
                    .completed_levels
                    .internal_set()
                    .contains(level.source.as_ref().unwrap());
                if completed_level || completed_previous_level {
                    let mouse_pos = window.mouse_position();
                    let mouse_pos = camera_transform
                        .inverse()
                        .transform_point(Vector2f::new(mouse_pos.x as f32, mouse_pos.y as f32));
                    if level_icon.global_bounds().contains(mouse_pos) {
                        if self.clicked {
                            let mut sound = Sound::with_buffer(&ctx.assets.ui_click_sound);
                            sound.set_volume(60.);
                            sound.play();
                            ctx.sound.add_sound(sound);
                            next_state = Some(Box::new(
                                Transitioning::new(
                                    ctx.assets,
                                    self.clone(),
                                    Playing::new(ctx, level_idx, level_array.category).unwrap(),
                                )
                                .unwrap(),
                            ));
                        }

                        self.level_hovered = Some((level_array.category, level_idx));
                    }
                }

                level_icon.move_(Vector2f::new(level_icon.global_bounds().width, 0.));

                completed_previous_level = completed_level;
            }
        }

        self.clicked = false;

        if let Some(next_state) = next_state {
            ControlFlow::Break(next_state)
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
            Event::MouseButtonReleased {
                button: sfml::window::mouse::Button::Left,
                ..
            } => {
                self.clicked = true;
            }
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
            let mut level_icon = ctx.assets.icon_tilesheet.tile_sprite(Gid(92)).unwrap();
            let category = &ctx.assets.level_categories[level_array.category];
            level_icon.set_position(Vector2f::new(level_array.rect.left, level_array.rect.top));
            level_icon.set_scale(Vector2f::new(
                level_array.rect.height / level_icon.global_bounds().height,
                level_array.rect.height / level_icon.global_bounds().height,
            ));

            let mut completed_previous_level = true;
            for (level_idx, level) in category.maps.iter().enumerate() {
                let completed_level = ctx
                    .completed_levels
                    .internal_set()
                    .contains(level.source.as_ref().unwrap());
                let mut color;
                if completed_level || completed_previous_level {
                    if matches!(self.level_hovered, Some((x, y)) if x == level_array.category && y == level_idx)
                    {
                        let amount_to_saturate = if sfml::window::mouse::Button::Left.is_pressed() {
                            60
                        } else {
                            30
                        };
                        color = category.color;
                        *color.red_mut() = color.red().saturating_add(amount_to_saturate);
                        *color.green_mut() = color.green().saturating_add(amount_to_saturate);
                        *color.blue_mut() = color.blue().saturating_add(amount_to_saturate);
                    } else {
                        color = category.color;
                    }
                } else {
                    color = category.color;
                    *color.alpha_mut() = 50;
                }
                level_icon.set_color(color);
                target.draw_with_renderstates(&level_icon, &render_states);

                level_icon.move_(Vector2f::new(level_icon.global_bounds().width, 0.));

                completed_previous_level = completed_level;
            }
        }
    }
}
