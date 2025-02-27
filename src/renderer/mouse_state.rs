#[derive(Debug, Default)]
pub struct MouseState {
    pressed: bool,
    position_x: f32,
    position_y: f32,
    start_drag: Option<(f32, f32)>,
}

impl MouseState {
    pub(crate) const fn pressed(&self) -> bool {
        self.pressed
    }

    pub(crate) fn set_pressed(&mut self, state: bool) {
        self.pressed = state;
    }

    pub(crate) const fn start_drag(&self) -> Option<(f32, f32)> {
        self.start_drag
    }

    pub(crate) const fn set_start_drag(&mut self, original_drag: Option<(f32, f32)>) {
        self.start_drag = original_drag;
    }

    pub(crate) const fn position(&self) -> (f32, f32) {
        (self.position_x, self.position_y)
    }

    pub(crate) fn update_position(&mut self, x: f32, y: f32) {
        self.position_x = x;
        self.position_y = y;
    }
}
