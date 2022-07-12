use std::time::Duration;

use sfml::{
    graphics::{Color, FloatRect, RenderTarget, RenderWindow, Text, Transformable},
    system::Vector2f,
};
use tiled::objects::ObjectShape;

use crate::{assets::AssetManager, level::Level};

pub struct LevelArray {
    pub rect: FloatRect,
    pub category: usize,
}

pub enum PlayState<'s> {
    LevelSelect {
        texts: Vec<Text<'s>>,
        level_arrays: Vec<LevelArray>,
        viewport_offset: Vector2f,
    },
    Playing {
        level: Level<'s>,
    },
    Transitioning {
        prev_level: Level<'s>,
        next_level: Level<'s>,
        time_left: Duration,
    },
}

impl<'s> PlayState<'s> {
    pub fn level_select(
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

        Self::LevelSelect {
            texts,
            level_arrays,
            viewport_offset,
        }
    }
}
