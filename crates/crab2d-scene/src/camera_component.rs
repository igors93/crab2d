#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera2DComponent {
    pub zoom: f32,
    pub clear_color: [f32; 4],
}

impl Camera2DComponent {
    pub const DEFAULT_CLEAR_COLOR: [f32; 4] = [0.08, 0.09, 0.10, 1.0];

    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        if zoom.is_finite() && zoom > 0.0 {
            self.zoom = zoom;
        }
        self
    }
}

impl Default for Camera2DComponent {
    fn default() -> Self {
        Self {
            zoom: 1.0,
            clear_color: Self::DEFAULT_CLEAR_COLOR,
        }
    }
}
