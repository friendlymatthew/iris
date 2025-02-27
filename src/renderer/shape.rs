#[derive(Debug)]
pub struct ShapeStack {
    shapes: Vec<Shape>,
}

impl ShapeStack {
    pub const fn new() -> Self {
        Self { shapes: vec![] }
    }

    pub fn push(&mut self, shape: Shape) {
        self.shapes.push(shape);
    }

    pub fn shapes(&self) -> &[Shape] {
        &self.shapes
    }
}

#[derive(Debug)]
pub enum Shape {
    Circle { x: f32, y: f32, radius: f32 },
}

pub fn compute_radius(from: (f32, f32), to: (f32, f32)) -> f32 {
    (from.0 - to.0).hypot(from.1 - to.1)
}
