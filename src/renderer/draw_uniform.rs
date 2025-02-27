#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DrawUniform {
    pub crosshair: u32,
    pub circle_center_x: f32,
    pub circle_center_y: f32,
    pub circle_radius: f32,
}

impl DrawUniform {
    pub const fn new() -> Self {
        Self {
            crosshair: 0,
            circle_center_x: 0.0,
            circle_center_y: 0.0,
            circle_radius: 0.0,
        }
    }

    pub(crate) const fn crosshair(&self) -> bool {
        self.crosshair == 1
    }

    pub(crate) fn toggle_crosshair(&mut self) {
        self.crosshair = !self.crosshair() as u32;
    }

    pub(crate) fn set_circle_center(&mut self, x: f32, y: f32) {
        self.circle_center_x = x;
        self.circle_center_y = y;
    }

    pub(crate) fn set_circle_radius(&mut self, radius: f32) {
        self.circle_radius = radius;
    }
}

impl DrawUniform {}
