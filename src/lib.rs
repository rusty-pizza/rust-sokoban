use assets::AssetManager;
use context::Context;
use level::Level;
use sfml::{
    graphics::{
        BlendMode, RenderStates, RenderTarget, RenderWindow, Text, Transform, Transformable,
    },
    system::{Vector2f, Vector2u},
    window::{ContextSettings, Event, Key, Style},
};
use sound_manager::SoundManager;

pub mod assets;
pub mod context;
pub mod graphics;
pub mod level;
pub mod sound_manager;

/// Run the game, returning on failure.
/// Will load and display the [`Level`] at [`LEVEL_PATH`].
pub fn run() -> anyhow::Result<()> {
    // Initialize
    let assets = AssetManager::load()?;
    let mut current_level_idx = 0;
    let mut level = Level::from_map(&assets.maps[0], &assets.tilesheet)?;
    let mut window = create_window();
    let mut sound = SoundManager::new();

    let mut last_frame_time = std::time::Instant::now();

    loop {
        let is_level_won = level.is_won();
        // Process events
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => return Ok(()),
                Event::KeyPressed { .. } if is_level_won => {
                    // Go to next level
                    current_level_idx += 1;

                    if current_level_idx >= assets::LEVEL_PATHS.len() {
                        println!("You won!");
                        return Ok(());
                    } else {
                        level =
                            Level::from_map(&assets.maps[current_level_idx], &assets.tilesheet)?;
                    }
                }
                Event::KeyPressed { code: Key::R, .. } => {
                    level = Level::from_map(&assets.maps[current_level_idx], &assets.tilesheet)?
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

        // Update
        let this_frame_time = std::time::Instant::now();
        let delta_time = this_frame_time - last_frame_time;

        level.update(
            Context {
                assets: &assets,
                sound: &mut sound,
            },
            delta_time,
        );
        sound.update();

        // Render frame
        let camera_transform = camera_transform(window.size(), level.tilemap().size());
        let render_states = RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

        window.clear(level.background_color);
        window.draw_with_renderstates(&level, &render_states);

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

        window.display();

        last_frame_time = this_frame_time;
    }
}

fn create_window() -> RenderWindow {
    const AA_LEVEL: u32 = 2;

    // Create the window of the application
    let mut context_settings = ContextSettings::default();
    context_settings.antialiasing_level = AA_LEVEL;
    let mut window = RenderWindow::new((1080, 720), "Sokoban!", Style::CLOSE, &context_settings);
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
