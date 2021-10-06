use sfml::{
    graphics::{Color, RenderStates, RenderTarget, RenderWindow, Sprite, Texture},
    window::{ContextSettings, Event, Style},
};

fn create_window() -> RenderWindow {
    const AA_LEVEL: u32 = 2;

    // Create the window of the application
    let mut context_settings = ContextSettings::default();
    context_settings.set_antialiasing_level(AA_LEVEL);
    let mut window = RenderWindow::new((1080, 720), "Sokoban!", Style::CLOSE, &context_settings);
    window.set_vertical_sync_enabled(true);

    window
}

pub fn run() {
    let mut window = create_window();
    let test_texture = Texture::from_file("assets/sprites/playerFace.png").expect("test texture");
    let sprite = Sprite::with_texture(&test_texture);

    loop {
        while let Some(event) = window.poll_event() {
            match event {
                Event::Closed => return,
                _ => (),
            }
        }

        window.clear(Color::BLACK);

        window.draw_sprite(&sprite, &RenderStates::DEFAULT);
        window.display();
    }
}
