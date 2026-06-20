#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyCode {
    Escape,
    Space,
    Enter,
    Character(char),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlatformEvent {
    CloseRequested,
    KeyPressed(KeyCode),
    CursorMoved { x: f32, y: f32 },
}

pub trait PlatformShell {
    fn poll_events(&mut self) -> Vec<PlatformEvent>;
}

#[derive(Debug, Default)]
pub struct HeadlessShell;

impl PlatformShell for HeadlessShell {
    fn poll_events(&mut self) -> Vec<PlatformEvent> {
        Vec::new()
    }
}
