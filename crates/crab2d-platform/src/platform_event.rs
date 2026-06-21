use crate::KeyCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Standard gamepad buttons (XInput layout).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GamepadButton {
    South,
    East,
    North,
    West,
    LeftBumper,
    RightBumper,
    LeftTrigger,
    RightTrigger,
    Select,
    Start,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    LeftStick,
    RightStick,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlatformEvent {
    CloseRequested,
    KeyPressed(KeyCode),
    KeyReleased(KeyCode),
    CursorMoved {
        x: f32,
        y: f32,
    },
    MouseButtonPressed(MouseButton),
    MouseButtonReleased(MouseButton),
    MouseScrolled {
        delta_x: f32,
        delta_y: f32,
    },
    GamepadButtonPressed(GamepadButton),
    GamepadButtonReleased(GamepadButton),
    /// Left stick (x, y) in range -1..1
    GamepadLeftStick {
        x: f32,
        y: f32,
    },
    /// Right stick (x, y) in range -1..1
    GamepadRightStick {
        x: f32,
        y: f32,
    },
}

impl PlatformEvent {
    pub fn is_close_requested(self) -> bool {
        matches!(self, Self::CloseRequested)
    }
}
