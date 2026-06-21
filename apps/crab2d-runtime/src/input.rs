use std::collections::BTreeSet;

use crab2d_platform::{InputState, KeyCode, MouseButton, PlatformEvent};
use eframe::egui;

pub(crate) fn collect_runtime_input(
    input: &mut InputState,
    previous_keys: &mut BTreeSet<KeyCode>,
    ctx: &egui::Context,
) {
    input.begin_frame();
    let current_keys = runtime_keys()
        .into_iter()
        .filter_map(|(egui_key, key_code)| {
            ctx.input(|input| input.key_down(egui_key))
                .then_some(key_code)
        })
        .collect::<BTreeSet<_>>();

    for key in current_keys.difference(previous_keys).copied() {
        input.apply_event(PlatformEvent::KeyPressed(key));
    }
    for key in previous_keys.difference(&current_keys).copied() {
        input.apply_event(PlatformEvent::KeyReleased(key));
    }
    if let Some(position) = ctx.input(|input| input.pointer.hover_pos()) {
        input.apply_event(PlatformEvent::CursorMoved {
            x: position.x,
            y: position.y,
        });
    }

    let mouse_buttons = [
        (egui::PointerButton::Primary, MouseButton::Left),
        (egui::PointerButton::Secondary, MouseButton::Right),
        (egui::PointerButton::Middle, MouseButton::Middle),
    ];
    for (egui_btn, btn) in mouse_buttons {
        if ctx.input(|input| input.pointer.button_pressed(egui_btn)) {
            input.apply_event(PlatformEvent::MouseButtonPressed(btn));
        }
        if ctx.input(|input| input.pointer.button_released(egui_btn)) {
            input.apply_event(PlatformEvent::MouseButtonReleased(btn));
        }
    }

    let scroll = ctx.input(|input| input.smooth_scroll_delta);
    if scroll.x != 0.0 || scroll.y != 0.0 {
        input.apply_event(PlatformEvent::MouseScrolled {
            delta_x: scroll.x,
            delta_y: scroll.y,
        });
    }

    *previous_keys = current_keys;
}

pub(crate) fn key_name(key: KeyCode) -> String {
    match key {
        KeyCode::Character(c) => c.to_string(),
        KeyCode::ArrowUp => "arrow_up".to_owned(),
        KeyCode::ArrowDown => "arrow_down".to_owned(),
        KeyCode::ArrowLeft => "arrow_left".to_owned(),
        KeyCode::ArrowRight => "arrow_right".to_owned(),
        KeyCode::Escape => "escape".to_owned(),
        KeyCode::Space => "space".to_owned(),
        KeyCode::Enter => "enter".to_owned(),
        KeyCode::F1 => "f1".to_owned(),
        KeyCode::F2 => "f2".to_owned(),
        KeyCode::F3 => "f3".to_owned(),
        KeyCode::F4 => "f4".to_owned(),
        KeyCode::F5 => "f5".to_owned(),
    }
}

fn runtime_keys() -> [(egui::Key, KeyCode); 14] {
    [
        (egui::Key::W, KeyCode::Character('w')),
        (egui::Key::A, KeyCode::Character('a')),
        (egui::Key::S, KeyCode::Character('s')),
        (egui::Key::D, KeyCode::Character('d')),
        (egui::Key::ArrowUp, KeyCode::ArrowUp),
        (egui::Key::ArrowDown, KeyCode::ArrowDown),
        (egui::Key::ArrowLeft, KeyCode::ArrowLeft),
        (egui::Key::ArrowRight, KeyCode::ArrowRight),
        (egui::Key::Escape, KeyCode::Escape),
        (egui::Key::Space, KeyCode::Space),
        (egui::Key::Enter, KeyCode::Enter),
        (egui::Key::F1, KeyCode::F1),
        (egui::Key::F2, KeyCode::F2),
        (egui::Key::F3, KeyCode::F3),
    ]
}
