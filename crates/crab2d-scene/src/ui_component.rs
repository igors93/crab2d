use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
pub enum UiAnchor {
    #[default]
    TopLeft,
    TopCenter,
    TopRight,
    MiddleLeft,
    Center,
    MiddleRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiLabelComponent {
    pub text: String,
    pub font_size: f32,
    pub color_rgba: [u8; 4],
    pub anchor: UiAnchor,
    pub offset_x: f32,
    pub offset_y: f32,
    pub visible: bool,
}

impl UiLabelComponent {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            font_size: 16.0,
            color_rgba: [255, 255, 255, 255],
            anchor: UiAnchor::TopLeft,
            offset_x: 0.0,
            offset_y: 0.0,
            visible: true,
        }
    }
    pub fn with_font_size(mut self, size: f32) -> Self {
        self.font_size = size;
        self
    }
    pub fn with_color(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.color_rgba = [r, g, b, a];
        self
    }
    pub fn with_anchor(mut self, anchor: UiAnchor) -> Self {
        self.anchor = anchor;
        self
    }
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UiPanelComponent {
    pub width: f32,
    pub height: f32,
    pub color_rgba: [u8; 4],
    pub anchor: UiAnchor,
    pub offset_x: f32,
    pub offset_y: f32,
    pub visible: bool,
}

impl UiPanelComponent {
    pub fn new(width: f32, height: f32) -> Self {
        Self {
            width,
            height,
            color_rgba: [0, 0, 0, 180],
            anchor: UiAnchor::TopLeft,
            offset_x: 0.0,
            offset_y: 0.0,
            visible: true,
        }
    }
    pub fn with_color(mut self, r: u8, g: u8, b: u8, a: u8) -> Self {
        self.color_rgba = [r, g, b, a];
        self
    }
    pub fn with_anchor(mut self, anchor: UiAnchor) -> Self {
        self.anchor = anchor;
        self
    }
    pub fn at(mut self, x: f32, y: f32) -> Self {
        self.offset_x = x;
        self.offset_y = y;
        self
    }
}
