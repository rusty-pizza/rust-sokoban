use std::{collections::HashSet, path::PathBuf, time::Duration};

use assets::AssetManager;
use context::Context;
use level::Level;
use sfml::{
    graphics::{
        BlendMode, Color, Rect, RectangleShape, RenderStates, RenderTarget, RenderWindow, Shape,
        Text, Transform, Transformable,
    },
    system::{Vector2f, Vector2u},
    window::{ContextSettings, Event, Key, Style},
};
use sound_manager::SoundManager;
use state::PlayState;
use tiled::tile::Gid;

pub mod assets;
pub mod context;
pub mod graphics;
pub mod level;
pub mod sound_manager;
pub mod state;

/// Run the game, returning on failure.
/// Will load and display the [`Level`] at [`LEVEL_PATH`].
pub fn run() -> anyhow::Result<()> {
    // Initialize
    let assets = AssetManager::load()?;
    let mut current_category_idx = 0;
    let mut current_level_idx = 0;
    let mut window = create_window();
    let mut sound = SoundManager::new();
    let mut state = PlayState::level_select(&assets, &window, 0);
    let mut completed_levels: HashSet<PathBuf> = HashSet::new();

    let mut last_frame_time = std::time::Instant::now();

    loop {
        const TRANSITION_TIME: Duration = Duration::from_secs(1);

        let this_frame_time = std::time::Instant::now();
        let delta_time = this_frame_time - last_frame_time;

        sound.update();

        match &mut state {
            PlayState::LevelSelect {
                texts,
                level_arrays,
                viewport_offset,
            } => {
                window.clear(
                    assets
                        .main_menu
                        .background_color
                        .map_or(Color::BLACK, |c| Color::rgb(c.red, c.green, c.blue)),
                );

                let mut clicked = false;
                let mut next_state: Option<PlayState> = None;

                // TODO: Handle resize event
                // Process events
                while let Some(event) = window.poll_event() {
                    match event {
                        Event::Closed => return Ok(()),
                        Event::MouseButtonReleased {
                            button: sfml::window::mouse::Button::Left,
                            ..
                        } => {
                            clicked = true;
                        }
                        Event::Resized { width, height } => {
                            let view = sfml::graphics::View::from_rect(&Rect {
                                left: 0.,
                                top: 0.,
                                width: width as f32,
                                height: height as f32,
                            });
                            window.set_view(&view);
                            next_state = Some(PlayState::level_select(
                                &assets,
                                &window,
                                completed_levels.len(),
                            ));
                        }

                        // Unlock all levels when Ctrl+I is pressed
                        Event::KeyPressed {
                            code: Key::I,
                            ctrl: true,
                            ..
                        } => {
                            for category in assets.level_categories.iter() {
                                for level in category.maps.iter() {
                                    completed_levels.insert(level.source.clone().unwrap());
                                }
                            }

                            next_state = Some(PlayState::level_select(
                                &assets,
                                &window,
                                completed_levels.len(),
                            ));
                        }
                        _ => (),
                    }
                }

                for text in texts {
                    window.draw(text);
                }

                for level_array in level_arrays {
                    let mut level_icon = assets.icon_tilesheet.tile_sprite(Gid(100)).unwrap();
                    let category = &assets.level_categories[level_array.category];
                    level_icon.set_position(
                        Vector2f::new(level_array.rect.left, level_array.rect.top)
                            + *viewport_offset,
                    );
                    level_icon.set_scale(Vector2f::new(
                        level_array.rect.height / level_icon.global_bounds().height,
                        level_array.rect.height / level_icon.global_bounds().height,
                    ));

                    let mut completed_previous_level = true;
                    for (level_idx, level) in category.maps.iter().enumerate() {
                        let completed_level =
                            completed_levels.contains(level.source.as_ref().unwrap());
                        let mut color;
                        if completed_level || completed_previous_level {
                            let mouse_pos = window.mouse_position();
                            if level_icon
                                .global_bounds()
                                .contains(Vector2f::new(mouse_pos.x as f32, mouse_pos.y as f32))
                            {
                                if clicked {
                                    next_state = Some(PlayState::Playing {
                                        level: Level::from_map(level, &assets.tilesheet)?,
                                    });
                                    current_category_idx = level_array.category;
                                    current_level_idx = level_idx;
                                }

                                let amount_to_saturate =
                                    if sfml::window::mouse::Button::Left.is_pressed() {
                                        60
                                    } else {
                                        30
                                    };
                                color = category.color;
                                *color.red_mut() = color.red().saturating_add(amount_to_saturate);
                                *color.green_mut() =
                                    color.green().saturating_add(amount_to_saturate);
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
                    state = next_state;
                }
            }
            PlayState::Playing { level } => {
                let is_level_won = level.is_won();

                // Update
                level.update(
                    Context {
                        assets: &assets,
                        sound: &mut sound,
                    },
                    delta_time,
                );

                // Render frame
                let camera_transform = camera_transform(window.size(), level.tilemap().size());
                let render_states =
                    RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

                window.clear(level.background_color);

                window.draw_with_renderstates(level, &render_states);

                if is_level_won {
                    let mut text = Text::new("Level complete!", &assets.win_font, 60);
                    text.set_position(Vector2f::new(
                        window.size().x as f32 / 2. - text.global_bounds().width / 2.,
                        10.,
                    ));
                    window.draw_with_renderstates(&text, &RenderStates::DEFAULT);
                    let mut subtext = Text::new("Press any key to continue", &assets.win_font, 30);
                    subtext.set_position(Vector2f::new(
                        window.size().x as f32 / 2. - subtext.global_bounds().width / 2.,
                        10. + text.global_bounds().height + 20.,
                    ));
                    window.draw_with_renderstates(&subtext, &RenderStates::DEFAULT);
                }

                let mut next_state: Option<PlayState> = None;
                // Process events
                while let Some(event) = window.poll_event() {
                    match event {
                        Event::Closed => return Ok(()),
                        Event::KeyPressed { .. } if is_level_won => {
                            // Mark this level as complete
                            completed_levels.insert(
                                assets.level_categories[current_category_idx].maps
                                    [current_level_idx]
                                    .source
                                    .clone()
                                    .unwrap(),
                            );

                            // Go to next level
                            current_level_idx += 1;

                            if current_level_idx
                                >= assets.level_categories[current_category_idx].maps.len()
                            {
                                current_level_idx = 0;
                                current_category_idx += 1;
                            }

                            if current_category_idx >= assets.level_categories.len() {
                                println!("You won!");
                                return Ok(());
                            } else {
                                next_state = Some(PlayState::Transitioning {
                                    prev_level: level.clone(),
                                    next_level: Level::from_map(
                                        &assets.level_categories[current_category_idx].maps
                                            [current_level_idx],
                                        &assets.tilesheet,
                                    )?,
                                    time_left: TRANSITION_TIME,
                                });
                            }
                        }
                        Event::KeyPressed {
                            code: Key::Escape, ..
                        } => {
                            next_state = Some(PlayState::level_select(
                                &assets,
                                &window,
                                completed_levels.len(),
                            ));
                        }
                        Event::KeyPressed { code: Key::R, .. } => {
                            *level = Level::from_map(
                                &assets.level_categories[current_category_idx].maps
                                    [current_level_idx],
                                &assets.tilesheet,
                            )?
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
                        _ => level.handle_event(
                            Context {
                                assets: &assets,
                                sound: &mut sound,
                            },
                            event,
                        ),
                    }
                }

                if let Some(next_state) = next_state {
                    state = next_state;
                }
            }

            PlayState::Transitioning {
                prev_level,
                next_level,
                time_left,
            } => {
                let is_fading_out = *time_left > TRANSITION_TIME / 2;

                let mut transition_color = prev_level.background_color;
                *transition_color.alpha_mut() = (255.
                    - ((time_left.as_secs_f32() / (TRANSITION_TIME.as_secs_f32() / 2.)) - 1.).abs()
                        * 255.) as u8;
                let current_level = if is_fading_out {
                    prev_level
                } else {
                    next_level
                };

                // Render frame
                let camera_transform =
                    camera_transform(window.size(), current_level.tilemap().size());
                let render_states =
                    RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

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

                // Process events
                while let Some(event) = window.poll_event() {
                    match event {
                        Event::Closed => return Ok(()),
                        _ => (),
                    }
                }

                // Update time left on transition
                *time_left = time_left.saturating_sub(delta_time);

                if time_left.is_zero() {
                    state = PlayState::Playing {
                        level: current_level.clone(),
                    };
                }
            }
        }

        window.display();

        last_frame_time = this_frame_time;
    }
}

fn create_window() -> RenderWindow {
    const AA_LEVEL: u32 = 2;

    // Create the window of the application
    let mut context_settings = ContextSettings::default();
    context_settings.antialiasing_level = AA_LEVEL;
    let mut window = RenderWindow::new(
        (1080, 720),
        "Sokoban!",
        Style::CLOSE | Style::RESIZE,
        &context_settings,
    );
    window.set_vertical_sync_enabled(true);

    window
}

fn camera_transform(window_size: Vector2u, map_size: Vector2u) -> Transform {
    const WINDOW_VERTICAL_PADDING: f32 = 200.0;
    let map_size = Vector2f::new(map_size.x as f32, map_size.y as f32);
    let window_size = Vector2f::new(window_size.x as f32, window_size.y as f32);
    let viewport_size = Vector2f::new(window_size.x, window_size.y - WINDOW_VERTICAL_PADDING);

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
