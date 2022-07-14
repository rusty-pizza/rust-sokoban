use std::ops::ControlFlow;

use sfml::{
    graphics::{FloatRect, Rect, RenderTarget, Transformable},
    window::{Event, Key},
};

use tiled::{self, tile::Gid};

use sfml::graphics::Color;

use tiled::objects::ObjectShape;

use sfml::graphics::RenderWindow;

use crate::{assets::AssetManager, context::Context, level::Level};

use sfml::system::Vector2f;

use super::{playing::Playing, LevelArray, State};

use sfml::graphics::Text;

pub struct LevelSelect<'s> {
    pub(crate) texts: Vec<Text<'s>>,
    pub(crate) level_arrays: Vec<LevelArray>,
    pub(crate) viewport_offset: Vector2f,
    pub(crate) clicked: bool,
}

impl<'s> LevelSelect<'s> {
    pub fn new(
        assets: &'s AssetManager,
        window: &RenderWindow,
        completed_level_count: usize,
    ) -> Self {
        let ui_aspect_ratio = assets.main_menu.width as f32 / assets.main_menu.height as f32;
        let target_aspect_ratio = window.size().x as f32 / window.size().y as f32;
        let window_size = window.size();
        let window_size = Vector2f::new(window_size.x as f32, window_size.y as f32);

        // Get the size of the viewport we will be actually projecting stuff onto
        let viewport_size = if ui_aspect_ratio > target_aspect_ratio {
            Vector2f::new(window_size.x, window_size.x / ui_aspect_ratio)
        } else {
            Vector2f::new(window_size.y * ui_aspect_ratio, window_size.y)
        };

        let viewport_offset = (window_size - viewport_size) / 2.;

        let map_scale =
            viewport_size.x / (assets.main_menu.width * assets.main_menu.tile_width) as f32;

        let mut texts = Vec::new();
        let mut level_arrays = Vec::new();

        for object in assets.main_menu.object_groups[0].objects.iter() {
            if let ObjectShape::Text {
                pixel_size,
                halign,
                valign,
                color,
                contents,
                ..
            } = &object.shape
            {
                let contents = if object.name == "level_metrics" {
                    format!("{}/{}", completed_level_count, assets.total_level_count())
                } else {
                    contents.clone()
                };
                let mut text = Text::new(
                    &contents,
                    &assets.win_font,
                    (*pixel_size as f32 * map_scale) as u32,
                );
                text.set_fill_color(Color::rgb(color.red, color.green, color.blue));
                let bounds = text.local_bounds();
                text.set_position(Vector2f::new(object.x * map_scale, object.y * map_scale));
                text.move_(Vector2f::new(
                    match halign {
                        tiled::objects::HorizontalAlignment::Left => -bounds.left,
                        tiled::objects::HorizontalAlignment::Center => {
                            object.width * map_scale / 2.
                                - text.local_bounds().width / 2.
                                - bounds.left
                        }
                        tiled::objects::HorizontalAlignment::Right => {
                            object.width * map_scale - text.local_bounds().width - bounds.left
                        }
                        tiled::objects::HorizontalAlignment::Justify => {
                            unimplemented!("Justified texts are not implemented")
                        }
                    },
                    match valign {
                        tiled::objects::VerticalAlignment::Top => -bounds.top,
                        tiled::objects::VerticalAlignment::Center => {
                            object.height * map_scale / 2.
                                - text.local_bounds().height / 2.
                                - bounds.top
                        }
                        tiled::objects::VerticalAlignment::Bottom => {
                            // FIXME: This is wrong! Bottom alignment should not depend on text bounds
                            // and instead should rely on font baseline and other characteristics.
                            // As SFML does not expose them, we are limited to this hack instead.
                            object.height * map_scale - bounds.height - bounds.top
                        }
                    },
                ));
                text.move_(viewport_offset);

                texts.push(text);
            } else if object.name == "level_array" {
                let rect = FloatRect::new(
                    object.x * map_scale,
                    object.y * map_scale,
                    object.width * map_scale,
                    object.height * map_scale,
                );
                let category = assets
                    .level_categories
                    .iter()
                    .enumerate()
                    .find(|(_, cat)| cat.name == object.obj_type)
                    .expect("Unknown level category in level map")
                    .0;
                level_arrays.push(LevelArray { rect, category });
            }
        }

        Self {
            texts,
            level_arrays,
            viewport_offset,
            clicked: false,
        }
    }
}

impl<'s> State<'s> for LevelSelect<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s, '_, '_>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        window.clear(
            ctx.assets
                .main_menu
                .background_color
                .map_or(Color::BLACK, |c| Color::rgb(c.red, c.green, c.blue)),
        );

        let mut next_state: Option<Box<dyn State<'s> + 's>> = None;

        for text in self.texts.iter() {
            window.draw(text);
        }

        for level_array in self.level_arrays.iter() {
            let mut level_icon = ctx.assets.icon_tilesheet.tile_sprite(Gid(100)).unwrap();
            let category = &ctx.assets.level_categories[level_array.category];
            level_icon.set_position(
                Vector2f::new(level_array.rect.left, level_array.rect.top) + self.viewport_offset,
            );
            level_icon.set_scale(Vector2f::new(
                level_array.rect.height / level_icon.global_bounds().height,
                level_array.rect.height / level_icon.global_bounds().height,
            ));

            let mut completed_previous_level = true;
            for (level_idx, level) in category.maps.iter().enumerate() {
                let completed_level = ctx
                    .completed_levels
                    .contains(level.source.as_ref().unwrap());
                let mut color;
                if completed_level || completed_previous_level {
                    let mouse_pos = window.mouse_position();
                    if level_icon
                        .global_bounds()
                        .contains(Vector2f::new(mouse_pos.x as f32, mouse_pos.y as f32))
                    {
                        if self.clicked {
                            next_state = Some(Box::new(Playing {
                                level: Level::from_map(level, &ctx.assets.tilesheet).unwrap(),
                            }));
                            *ctx.current_category_idx = level_array.category;
                            *ctx.current_level_idx = level_idx;
                        }

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
                window.draw(&level_icon);

                level_icon.move_(Vector2f::new(level_icon.global_bounds().width, 0.));

                completed_previous_level = completed_level;
            }
        }

        if let Some(next_state) = next_state {
            ControlFlow::Break(next_state)
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

                *self = LevelSelect::new(ctx.assets, window, ctx.completed_levels.len());
            }

            // Unlock all levels when Ctrl+I is pressed
            Event::KeyPressed {
                code: Key::I,
                ctrl: true,
                ..
            } => {
                for category in ctx.assets.level_categories.iter() {
                    for level in category.maps.iter() {
                        ctx.completed_levels.insert(level.source.clone().unwrap());
                    }
                }

                *self = LevelSelect::new(ctx.assets, window, ctx.completed_levels.len());
            }
            _ => (),
        }

        ControlFlow::Continue(())
    }
}
