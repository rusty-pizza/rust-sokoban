use sfml::window::mouse;

pub struct InputSystem {
    clicked_this_frame: bool,
    clicked_last_frame: bool,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            clicked_this_frame: false,
            clicked_last_frame: false,
        }
    }

    pub fn update(&mut self) {
        self.clicked_last_frame = self.clicked_this_frame;
        self.clicked_this_frame = mouse::Button::Left.is_pressed()
    }

    pub fn just_pressed_lmb(&self) -> bool {
        self.clicked_this_frame && !self.clicked_last_frame
    }

    pub fn just_released_lmb(&self) -> bool {
        !self.clicked_this_frame && self.clicked_last_frame
    }
}
