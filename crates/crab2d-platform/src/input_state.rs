use std::collections::BTreeSet;

use crate::{GamepadButton, KeyCode, MouseButton, PlatformEvent};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct InputState {
    pressed: BTreeSet<KeyCode>,
    just_pressed: BTreeSet<KeyCode>,
    just_released: BTreeSet<KeyCode>,
    mouse_pressed: BTreeSet<MouseButton>,
    mouse_just_pressed: BTreeSet<MouseButton>,
    mouse_just_released: BTreeSet<MouseButton>,
    cursor_position: Option<(f32, f32)>,
    scroll_delta: (f32, f32),
    gamepad_pressed: BTreeSet<GamepadButton>,
    gamepad_just_pressed: BTreeSet<GamepadButton>,
    gamepad_just_released: BTreeSet<GamepadButton>,
    gamepad_left_stick: (f32, f32),
    gamepad_right_stick: (f32, f32),
}

impl InputState {
    pub fn begin_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
        self.mouse_just_pressed.clear();
        self.mouse_just_released.clear();
        self.scroll_delta = (0.0, 0.0);
        self.gamepad_just_pressed.clear();
        self.gamepad_just_released.clear();
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
            PlatformEvent::MouseButtonPressed(btn) => {
                if self.mouse_pressed.insert(btn) {
                    self.mouse_just_pressed.insert(btn);
                }
            }
            PlatformEvent::MouseButtonReleased(btn) => {
                if self.mouse_pressed.remove(&btn) {
                    self.mouse_just_released.insert(btn);
                }
            }
            PlatformEvent::MouseScrolled { delta_x, delta_y } => {
                self.scroll_delta.0 += delta_x;
                self.scroll_delta.1 += delta_y;
            }
            PlatformEvent::GamepadButtonPressed(btn) => {
                if self.gamepad_pressed.insert(btn) {
                    self.gamepad_just_pressed.insert(btn);
                }
            }
            PlatformEvent::GamepadButtonReleased(btn) => {
                if self.gamepad_pressed.remove(&btn) {
                    self.gamepad_just_released.insert(btn);
                }
            }
            PlatformEvent::GamepadLeftStick { x, y } => {
                self.gamepad_left_stick = (x, y);
            }
            PlatformEvent::GamepadRightStick { x, y } => {
                self.gamepad_right_stick = (x, y);
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

    pub fn is_mouse_down(&self, button: MouseButton) -> bool {
        self.mouse_pressed.contains(&button)
    }

    pub fn was_mouse_pressed(&self, button: MouseButton) -> bool {
        self.mouse_just_pressed.contains(&button)
    }

    pub fn was_mouse_released(&self, button: MouseButton) -> bool {
        self.mouse_just_released.contains(&button)
    }

    pub fn cursor_position(&self) -> Option<(f32, f32)> {
        self.cursor_position
    }

    pub fn scroll_delta(&self) -> (f32, f32) {
        self.scroll_delta
    }

    pub fn pressed_keys(&self) -> impl Iterator<Item = KeyCode> + '_ {
        self.pressed.iter().copied()
    }

    pub fn is_gamepad_down(&self, button: GamepadButton) -> bool {
        self.gamepad_pressed.contains(&button)
    }

    pub fn was_gamepad_pressed(&self, button: GamepadButton) -> bool {
        self.gamepad_just_pressed.contains(&button)
    }

    pub fn was_gamepad_released(&self, button: GamepadButton) -> bool {
        self.gamepad_just_released.contains(&button)
    }

    pub fn gamepad_left_stick(&self) -> (f32, f32) {
        self.gamepad_left_stick
    }

    pub fn gamepad_right_stick(&self) -> (f32, f32) {
        self.gamepad_right_stick
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
