#[repr(C)]
#[derive(Debug, Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FeatureUniform {
    grayscale: u32,
    // sepia: u32,
    invert: u32,
    gamma: u32,
    blur: u32,
    blur_radius: u32,
    width: u32,
    height: u32,
    sharpen: u32,
    sharpen_factor: u32,
    edge_detect: u32,
    _padding_1: [u8; 8],
    transform: TransformMatrix,
}

impl FeatureUniform {
    pub(crate) const fn new(width: u32, height: u32, gamma: u32) -> Self {
        Self {
            grayscale: 0,
            // sepia: 0,
            invert: 0,
            gamma,
            width,
            height,
            blur: 0,
            blur_radius: Self::DEFAULT_BLUR_RADIUS,
            sharpen: 0,
            sharpen_factor: Self::DEFAULT_SHARPEN_FACTOR,
            edge_detect: 0,
            _padding_1: [0u8; 8],
            transform: Self::TRANSFORM_IDENTITY,
        }
    }

    pub(crate) fn reset_features(&mut self) {
        self.grayscale = 0;
        // self.sepia = 0;
        self.invert = 0;
        self.blur = 0;
        self.sharpen = 0;
        self.edge_detect = 0;
        self.transform = Self::TRANSFORM_IDENTITY;
    }
}

impl FeatureUniform {
    pub(crate) const fn grayscale(&self) -> bool {
        self.grayscale == 1
    }

    // pub(crate) const fn sepia(&self) -> bool {
    //     self.sepia == 1
    // }

    pub(crate) const fn invert(&self) -> bool {
        self.invert == 1
    }

    pub(crate) fn toggle_grayscale(&mut self) {
        self.grayscale = !self.grayscale() as u32;
    }

    // pub(crate) fn toggle_sepia(&mut self) {
    //     self.sepia = !self.sepia() as u32;
    // }

    pub(crate) fn toggle_invert(&mut self) {
        self.invert = !self.invert() as u32;
    }
}

impl FeatureUniform {
    const DEFAULT_BLUR_RADIUS: u32 = 21;
    const MAX_BLUR_RADIUS: u32 = 39;
    const MIN_BLUR_RADIUS: u32 = 3;

    pub(crate) const fn blur(&self) -> bool {
        self.blur == 1
    }

    pub(crate) fn toggle_blur(&mut self) {
        self.blur = !self.blur() as u32;
    }

    pub(crate) fn increase_blur_radius(&mut self) {
        self.blur_radius = (self.blur_radius + 2).min(Self::MAX_BLUR_RADIUS);
    }

    pub(crate) fn decrease_blur_radius(&mut self) {
        self.blur_radius = (self.blur_radius - 2).max(Self::MIN_BLUR_RADIUS);
    }
}

impl FeatureUniform {
    const DEFAULT_SHARPEN_FACTOR: u32 = 16;
    const MAX_SHARPEN_FACTOR: u32 = 40;
    const MIN_SHARPEN_FACTOR: u32 = 1;

    pub(crate) const fn sharpen(&self) -> bool {
        self.sharpen == 1
    }

    pub(crate) fn toggle_sharpen(&mut self) {
        self.sharpen = !self.sharpen() as u32;
    }

    pub(crate) fn increase_sharpen_factor(&mut self) {
        self.sharpen_factor = (self.sharpen_factor + 1).min(Self::MAX_SHARPEN_FACTOR);
    }

    pub(crate) fn decrease_sharpen_factor(&mut self) {
        self.sharpen_factor = (self.sharpen_factor - 1).max(Self::MIN_SHARPEN_FACTOR);
    }
}

impl FeatureUniform {
    pub(crate) fn update_window_dimensions(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }
}

impl FeatureUniform {
    pub(crate) const fn edge_detect(&self) -> bool {
        self.edge_detect == 1
    }

    pub(crate) fn toggle_edge_detect(&mut self) {
        self.edge_detect = !self.edge_detect() as u32;
    }
}

type TransformMatrix = [[f32; 4]; 4];

#[derive(Debug, PartialEq, Eq)]
pub enum TransformAction {
    FlipX,
    FlipY,
}

impl TransformAction {
    const fn matrix(self) -> TransformMatrix {
        match self {
            Self::FlipX => FeatureUniform::TRANSFORM_FLIP_X,
            Self::FlipY => FeatureUniform::TRANSFORM_FLIP_Y,
        }
    }
}

impl FeatureUniform {
    const TRANSFORM_IDENTITY: TransformMatrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    const TRANSFORM_FLIP_X: TransformMatrix = [
        [-1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    const TRANSFORM_FLIP_Y: TransformMatrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, -1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];

    pub(crate) fn apply_transform(&mut self, action: TransformAction) {
        let transform_mat = action.matrix();

        for (r, row) in transform_mat.iter().enumerate() {
            for (c, ch) in row.iter().enumerate() {
                if r == 3 || c == 3 {
                    continue;
                }

                self.transform[r][c] *= ch;
            }
        }
    }
}
