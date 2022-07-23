use std::ops::ControlFlow;

use sfml::{
    audio::{Sound, SoundSource},
    graphics::{
        BlendMode, Drawable, FloatRect, Rect, RenderStates, RenderTarget, Transform, Transformable,
    },
    system::Vector2u,
    window::{Event, Key},
};

use tiled::{self, tile::Gid};

use sfml::graphics::Color;

use tiled::objects::ObjectShape;

use sfml::graphics::RenderWindow;

use crate::{assets::AssetManager, context::Context};

use sfml::system::Vector2f;

use super::{playing::Playing, LevelArray, State};

use sfml::graphics::Text;

pub struct LevelSelect<'s> {
    pub(crate) drawables: Vec<Box<dyn Drawable + 's>>,
    pub(crate) level_arrays: Vec<LevelArray>,
    pub(crate) clicked: bool,
    level_hovered: Option<(usize, usize)>,
}

impl<'s> LevelSelect<'s> {
    pub fn new(assets: &'s AssetManager, completed_level_count: usize) -> Self {
        let mut drawables: Vec<Box<dyn Drawable + 's>> = Vec::new();
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
                let mut text = Text::new(&contents, &assets.win_font, *pixel_size as u32);
                text.set_fill_color(Color::rgb(color.red, color.green, color.blue));
                let bounds = text.local_bounds();
                text.set_position(Vector2f::new(object.x, object.y));
                text.move_(Vector2f::new(
                    match halign {
                        tiled::objects::HorizontalAlignment::Left => -bounds.left,
                        tiled::objects::HorizontalAlignment::Center => {
                            object.width / 2. - text.local_bounds().width / 2. - bounds.left
                        }
                        tiled::objects::HorizontalAlignment::Right => {
                            object.width - text.local_bounds().width - bounds.left
                        }
                        tiled::objects::HorizontalAlignment::Justify => {
                            unimplemented!("Justified texts are not implemented")
                        }
                    },
                    match valign {
                        tiled::objects::VerticalAlignment::Top => -bounds.top,
                        tiled::objects::VerticalAlignment::Center => {
                            object.height / 2. - text.local_bounds().height / 2. - bounds.top
                        }
                        tiled::objects::VerticalAlignment::Bottom => {
                            // FIXME: This is wrong! Bottom alignment should not depend on text bounds
                            // and instead should rely on font baseline and other characteristics.
                            // As SFML does not expose them, we are limited to this hack instead.
                            object.height - bounds.height - bounds.top
                        }
                    },
                ));

                drawables.push(Box::new(text));
            } else if object.gid != Gid::EMPTY {
                let gid_tileset = assets
                    .main_menu
                    .tileset_by_gid(object.gid)
                    .expect("object in main menu has invalid gid");
                let tilesheet = match gid_tileset.name.as_str() {
                    "icons" => &assets.icon_tilesheet,
                    "Sokoban" => &assets.tilesheet,
                    _ => panic!("invalid tilesheet name for tile object found in main menu"),
                };
                let mut sprite = tilesheet
                    .tile_sprite(Gid(object.gid.0 - gid_tileset.first_gid.0 + 1))
                    .expect("invalid gid found in overlay object");
                sprite.set_scale(Vector2f::new(
                    object.width / sprite.texture_rect().width as f32,
                    object.height / sprite.texture_rect().height as f32,
                ));
                sprite.set_position(Vector2f::new(object.x, object.y));
                sprite.set_rotation(object.rotation);
                drawables.push(Box::new(sprite));
            } else if object.name == "level_array" {
                let rect = FloatRect::new(object.x, object.y, object.width, object.height);
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
            drawables,
            level_arrays,
            clicked: false,
            level_hovered: None,
        }
    }
}

impl<'s> State<'s> for LevelSelect<'s> {
    fn tick(
        &mut self,
        ctx: &mut Context<'s, '_, '_>,
        window: &mut RenderWindow,
    ) -> ControlFlow<Box<dyn State<'s> + 's>, ()> {
        let mut next_state: Option<Box<dyn State<'s> + 's>> = None;
        let camera_transform = camera_transform(
            window.size(),
            Vector2u::new(
                ctx.assets.main_menu.width * ctx.assets.main_menu.tile_width,
                ctx.assets.main_menu.height * ctx.assets.main_menu.tile_height,
            ),
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
                                Playing::new(ctx.assets, level_idx, level_array.category).unwrap(),
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

                *self = LevelSelect::new(ctx.assets, ctx.completed_levels.len());
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

                *self = LevelSelect::new(ctx.assets, ctx.completed_levels.len());
            }
            _ => (),
        }

        ControlFlow::Continue(())
    }

    fn draw(&self, ctx: &mut Context<'s, '_, '_>, target: &mut dyn RenderTarget) {
        let camera_transform = camera_transform(
            target.size(),
            Vector2u::new(
                ctx.assets.main_menu.width * ctx.assets.main_menu.tile_width,
                ctx.assets.main_menu.height * ctx.assets.main_menu.tile_height,
            ),
        );
        let render_states = RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

        target.clear(
            ctx.assets
                .main_menu
                .background_color
                .map_or(Color::BLACK, |c| Color::rgb(c.red, c.green, c.blue)),
        );

        for drawable in self.drawables.iter() {
            target.draw_with_renderstates(drawable.as_ref(), &render_states);
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

pub fn camera_transform(window_size: Vector2u, map_size: Vector2u) -> Transform {
    let map_size = Vector2f::new(map_size.x as f32, map_size.y as f32);
    let window_size = Vector2f::new(window_size.x as f32, window_size.y as f32);
    let viewport_size = Vector2f::new(window_size.x, window_size.y);

    let scale_factors = map_size / viewport_size;
    let map_scale = if scale_factors.x > scale_factors.y {
        scale_factors.x
    } else {
        scale_factors.y
    };
    let map_px_size = map_size / map_scale;

    let mut x = Transform::IDENTITY;
    x.scale_with_center(map_scale, map_scale, 0f32, 0f32);
    x.translate(
        (map_px_size.x - viewport_size.x) / 2f32 + (viewport_size.x - window_size.x) / 2f32,
        (map_px_size.y - viewport_size.y) / 2f32 + (viewport_size.y - window_size.y) / 2f32,
    );
    x.inverse()
}
