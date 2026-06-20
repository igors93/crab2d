use crate::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlatformEvent {
    CloseRequested,
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    CursorMoved { x: f32, y: f32 },
}

impl PlatformEvent {
    pub fn is_close_requested(self) -> bool {
        matches!(self, Self::CloseRequested)
    }
}
