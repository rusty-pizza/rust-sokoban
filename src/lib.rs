use std::{collections::HashSet, ops::ControlFlow, path::PathBuf};

use assets::AssetManager;
use context::Context;

use sfml::{
    graphics::RenderWindow,
    window::{ContextSettings, Event, Style},
};
use sound_manager::SoundManager;
use state::{LevelSelect, State};

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
    let mut state: Box<dyn State> = Box::new(LevelSelect::new(&assets, &window, 0));
    let mut completed_levels: HashSet<PathBuf> = HashSet::new();

    let mut last_frame_time = std::time::Instant::now();

    'outer: loop {
        let this_frame_time = std::time::Instant::now();
        let delta_time = this_frame_time - last_frame_time;

        sound.update();

        let mut context = Context {
            assets: &assets,
            completed_levels: &mut completed_levels,
            current_category_idx: &mut current_category_idx,
            current_level_idx: &mut current_level_idx,
            delta_time,
            sound: &mut sound,
        };

        if let ControlFlow::Break(new_state) = state.tick(&mut context, &mut window) {
            state = new_state;
        }

        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => break 'outer,
                _ => (),
            }

            if let ControlFlow::Break(new_state) =
                state.process_event(&mut context, &mut window, event)
            {
                state = new_state;
            }
        }

        window.display();

        last_frame_time = this_frame_time;
    }

    Ok(())
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
