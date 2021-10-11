use std::{array::IntoIter, collections::HashMap, iter::FromIterator, path::Path};

use level::Level;
use sfml::{
    graphics::{BlendMode, Color, RenderStates, RenderTarget, RenderWindow, Transform},
    system::{Vector2f, Vector2u},
    window::{ContextSettings, Event, Style},
};
use tilesheet::Tilesheet;

mod level;
mod quadarray;
mod tilesheet;

pub fn run() -> anyhow::Result<()> {
    let map = tiled::parse_file(Path::new("assets/levels/untitled.tmx"))?;
    let tilesheet = Tilesheet::from_file(Path::new("assets/tilesheets/sokoban_tilesheet.tsx"))?;
    let level = {
        Level::new(
            &map,
            &tilesheet,
            HashMap::from_iter(IntoIter::new([
                (7u32, (level::ObjectType::Crate, level::CrateStyle::Wooden)),
                (
                    40u32,
                    (level::ObjectType::CrateGoal, level::CrateStyle::Wooden),
                ),
            ])),
        )?
    };

    let mut window = create_window();

    loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => return Ok(()),
                _ => (),
            }
        }

        let camera_transform = camera_transform(window.size(), level.size());
        let render_states = RenderStates::new(BlendMode::ALPHA, camera_transform, None, None);

        let bg_color = map
            .background_colour
            .and_then(|c| Some(Color::rgb(c.red, c.green, c.blue)))
            .unwrap_or(Color::BLACK);
        window.clear(bg_color);
        window.draw_with_renderstates(&level, &render_states);
        window.display();
    }
}

fn create_window() -> RenderWindow {
    const AA_LEVEL: u32 = 2;

    // Create the window of the application
    let mut context_settings = ContextSettings::default();
    context_settings.set_antialiasing_level(AA_LEVEL);
    let mut window = RenderWindow::new((1080, 720), "Sokoban!", Style::CLOSE, &context_settings);
    window.set_vertical_sync_enabled(true);

    window
}

fn camera_transform(window_size: Vector2u, map_size: Vector2u) -> Transform {
    const WINDOW_VERTICAL_PADDING: f32 = 200.0;
    let mut x = Transform::IDENTITY;
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
    x.scale_with_center(map_scale, map_scale, 0f32, 0f32);
    x.translate(
        (map_px_size.x - viewport_size.x) / 2f32 + (viewport_size.x - window_size.x) / 2f32,
        (map_px_size.y - viewport_size.y) / 2f32 + (viewport_size.y - window_size.y) / 2f32,
    );
    x.inverse()
}
