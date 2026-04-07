use raylib::prelude::*;

pub enum ClipboardAction {
    /// Buffer was replaced with a new value.
    Replace(String),
    /// Ctrl+A was pressed — caller should enter select-all mode.
    SelectAll,
    /// Nothing clipboard-related happened.
    Noop,
}

/// Handles Ctrl/Cmd+C, Ctrl/Cmd+V, Ctrl/Cmd+X, Ctrl/Cmd+A for a text field.
/// `filter` is applied to each pasted character.
pub fn handle(rl: &mut RaylibHandle, buf: &str, filter: fn(char) -> bool) -> ClipboardAction {
    if !mod_down(rl) {
        return ClipboardAction::Noop;
    }

    if rl.is_key_pressed(KeyboardKey::KEY_C) {
        let _ = rl.set_clipboard_text(buf);
        return ClipboardAction::Noop;
    }

    if rl.is_key_pressed(KeyboardKey::KEY_X) {
        let _ = rl.set_clipboard_text(buf);
        return ClipboardAction::Replace(String::new());
    }

    if rl.is_key_pressed(KeyboardKey::KEY_A) {
        return ClipboardAction::SelectAll;
    }

    if rl.is_key_pressed(KeyboardKey::KEY_V) {
        if let Ok(text) = rl.get_clipboard_text() {
            let filtered: String = text.chars().filter(|&c| filter(c)).collect();
            if !filtered.is_empty() {
                return ClipboardAction::Replace(format!("{buf}{filtered}"));
            }
        }
    }

    ClipboardAction::Noop
}

fn mod_down(rl: &RaylibHandle) -> bool {
    rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
        || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)
        || rl.is_key_down(KeyboardKey::KEY_LEFT_SUPER)
        || rl.is_key_down(KeyboardKey::KEY_RIGHT_SUPER)
}
