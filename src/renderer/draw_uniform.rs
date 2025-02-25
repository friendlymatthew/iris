#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawUniform {
    crosshair: u32,
    drag: u32,
    drag_start_x: f32,
    drag_start_y: f32,
    drag_radius: f32,
}

impl DrawUniform {
    pub fn new() -> Self {
        Self {
            crosshair: 0,
            drag: 0,
            drag_start_x: 0.0,
            drag_start_y: 0.0,
            drag_radius: 0.0,
        }
    }

    pub(crate) const fn crosshair(&self) -> bool {
        self.crosshair == 1
    }

    pub(crate) fn toggle_crosshair(&mut self) {
        self.crosshair = !self.crosshair() as u32;
    }
}

impl DrawUniform {
    pub(crate) fn set_drag(&mut self, state: bool) {
        self.drag = state as u32;
    }

    pub(crate) fn set_start_drag_position(&mut self, x: f32, y: f32) {
        self.drag_start_x = x;
        self.drag_start_y = y;
    }

    pub(crate) fn compute_drag_radius(&mut self, x: f32, y: f32) {
        self.drag_radius = (self.drag_start_x - x).hypot(self.drag_start_y - y);
    }
}
