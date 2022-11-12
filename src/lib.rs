use std::{ops::ControlFlow, time::Duration};

use assets::AssetManager;
use context::{Context, SaveData};

#[cfg(feature = "editor")]
use guiedit::sfml::graphics::RenderWindow;
#[cfg(not(feature = "editor"))]
use sfml::graphics::RenderWindow;

use input_system::InputSystem;
use sfml::window::{ContextSettings, Event, Style};
use sound_manager::SoundManager;
use state::{LevelSelect, State};

pub mod assets;
pub mod context;
pub mod graphics;
pub mod input_system;
pub mod level;
pub mod sound_manager;
pub mod state;
pub mod ui;

/// Run the game, returning on failure.
/// Will load and display the [`Level`] at [`LEVEL_PATH`].
pub fn run() -> anyhow::Result<()> {
    env_logger::init();

    let assets = AssetManager::load()?;
    let mut window = create_window();
    let sound = SoundManager::new();
    let completed_levels = match SaveData::from_savefile() {
        Ok(x) => x,
        Err(err) => {
            log::warn!("could not load savefile: {}", err);
            Default::default()
        }
    };
    let input = InputSystem::new();

    let mut context = Context {
        assets: &assets,
        completed_levels,
        delta_time: Duration::default(),
        sound,
        input,
    };
    let mut state: Box<dyn State> = Box::new(LevelSelect::new(&context)?);

    let mut last_frame_time = std::time::Instant::now();
    'outer: loop {
        let this_frame_time = std::time::Instant::now();
        context.delta_time = this_frame_time - last_frame_time;

        context.sound.update();
        context.input.update(&window);

        if let ControlFlow::Break(new_state) = state.tick(&mut context, &mut window) {
            state = new_state;
        }

        while let Some(event) = window.poll_event() {
            if event == Event::Closed {
                break 'outer;
            }

            if let ControlFlow::Break(new_state) =
                state.process_event(&mut context, &mut window, event)
            {
                state = new_state;
            }
        }

        state.draw(&mut context, &mut window);

        #[cfg(feature = "editor")]
        window.display_and_inspect(&mut state);
        #[cfg(not(feature = "editor"))]
        window.display();

        last_frame_time = this_frame_time;
    }

    Ok(())
}

fn create_window() -> RenderWindow {
    // Create the window of the application
    let context_settings = ContextSettings::default();
    let mut window = RenderWindow::new(
        (1080, 720),
        "Sokoban!",
        Style::CLOSE | Style::RESIZE,
        &context_settings,
    );
    window.set_vertical_sync_enabled(true);

    window
}
