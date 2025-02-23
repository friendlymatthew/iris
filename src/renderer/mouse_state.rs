#[derive(Debug, Default)]
pub struct MouseState {
    pressed: bool,
    position_x: f32,
    position_y: f32,
}

impl MouseState {
    pub(crate) fn pressed(&self) -> bool {
        self.pressed
    }

    pub(crate) fn set_pressed(&mut self, state: bool) {
        self.pressed = state;
    }

    pub(crate) fn position(&self) -> (f32, f32) {
        (self.position_x, self.position_y)
    }

    pub(crate) fn update_position(&mut self, x: f32, y: f32) {
        self.position_x = x;
        self.position_y = y;
    }
}
