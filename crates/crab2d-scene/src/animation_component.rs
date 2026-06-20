use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationState {
    pub name: String,
    pub frames: Vec<u32>,
    pub fps: f32,
    pub looping: bool,
}

impl AnimationState {
    pub fn new(name: impl Into<String>, frames: Vec<u32>, fps: f32) -> Self {
        Self {
            name: name.into(),
            frames,
            fps,
            looping: true,
        }
    }
    pub fn once(mut self) -> Self {
        self.looping = false;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnimationComponent {
    pub spritesheet_path: String,
    pub frame_width: u32,
    pub frame_height: u32,
    pub columns: u32,
    pub states: Vec<AnimationState>,
    pub current_state: String,
    pub current_frame: u32,
    pub frame_timer: f32,
    pub playing: bool,
}

impl AnimationComponent {
    pub fn new(
        spritesheet_path: impl Into<String>,
        frame_width: u32,
        frame_height: u32,
        columns: u32,
    ) -> Self {
        Self {
            spritesheet_path: spritesheet_path.into(),
            frame_width,
            frame_height,
            columns,
            states: Vec::new(),
            current_state: String::new(),
            current_frame: 0,
            frame_timer: 0.0,
            playing: true,
        }
    }

    pub fn add_state(mut self, state: AnimationState) -> Self {
        if self.current_state.is_empty() {
            self.current_state = state.name.clone();
        }
        self.states.push(state);
        self
    }

    pub fn set_state(&mut self, name: &str) {
        if self.current_state != name {
            self.current_state = name.to_string();
            self.current_frame = 0;
            self.frame_timer = 0.0;
        }
    }

    pub fn current_tile_index(&self) -> u32 {
        if let Some(state) = self.states.iter().find(|s| s.name == self.current_state) {
            state
                .frames
                .get(self.current_frame as usize)
                .copied()
                .unwrap_or(0)
        } else {
            0
        }
    }

    /// Returns (u, v, u2, v2) UV coords for the current frame in the spritesheet
    pub fn current_uv(&self, sheet_width: u32, sheet_height: u32) -> (f32, f32, f32, f32) {
        if self.frame_width == 0 || self.frame_height == 0 || sheet_width == 0 || sheet_height == 0
        {
            return (0.0, 0.0, 1.0, 1.0);
        }
        let tile = self.current_tile_index();
        let cols = self.columns.max(1);
        let col = tile % cols;
        let row = tile / cols;
        let u = col as f32 * self.frame_width as f32 / sheet_width as f32;
        let v = row as f32 * self.frame_height as f32 / sheet_height as f32;
        let u2 = u + self.frame_width as f32 / sheet_width as f32;
        let v2 = v + self.frame_height as f32 / sheet_height as f32;
        (u, v, u2, v2)
    }
}
