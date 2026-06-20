use std::collections::BTreeSet;

use crate::{KeyCode, PlatformEvent};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputState {
    pressed: BTreeSet<KeyCode>,
    just_pressed: BTreeSet<KeyCode>,
    just_released: BTreeSet<KeyCode>,
    cursor_position: Option<(f32, f32)>,
}

impl InputState {
    pub fn begin_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn apply_event(&mut self, event: PlatformEvent) {
        match event {
            PlatformEvent::KeyPressed(key) => {
                if self.pressed.insert(key) {
                    self.just_pressed.insert(key);
                }
            }
            PlatformEvent::KeyReleased(key) => {
                if self.pressed.remove(&key) {
                    self.just_released.insert(key);
                }
            }
            PlatformEvent::CursorMoved { x, y } => {
                self.cursor_position = Some((x, y));
            }
            PlatformEvent::CloseRequested => {}
        }
    }

    pub fn apply_events(&mut self, events: impl IntoIterator<Item = PlatformEvent>) {
        for event in events {
            self.apply_event(event);
        }
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.pressed.contains(&key)
    }

    pub fn was_key_pressed(&self, key: KeyCode) -> bool {
        self.just_pressed.contains(&key)
    }

    pub fn was_key_released(&self, key: KeyCode) -> bool {
        self.just_released.contains(&key)
    }

    pub fn cursor_position(&self) -> Option<(f32, f32)> {
        self.cursor_position
    }

    pub fn pressed_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.pressed.iter().copied()
    }
}

#[cfg(test)]
mod tests {
    use crate::{InputState, KeyCode, PlatformEvent};

    #[test]
    fn input_tracks_pressed_and_released_keys() {
        let mut input = InputState::default();

        input.apply_event(PlatformEvent::KeyPressed(KeyCode::Space));

        assert!(input.is_key_down(KeyCode::Space));
        assert!(input.was_key_pressed(KeyCode::Space));
        assert!(!input.was_key_released(KeyCode::Space));

        input.begin_frame();
        assert!(input.is_key_down(KeyCode::Space));
        assert!(!input.was_key_pressed(KeyCode::Space));

        input.apply_event(PlatformEvent::KeyReleased(KeyCode::Space));
        assert!(!input.is_key_down(KeyCode::Space));
        assert!(input.was_key_released(KeyCode::Space));
    }

    #[test]
    fn input_tracks_cursor_position() {
        let mut input = InputState::default();

        input.apply_event(PlatformEvent::CursorMoved { x: 12.0, y: 24.0 });

        assert_eq!(input.cursor_position(), Some((12.0, 24.0)));
    }
}
