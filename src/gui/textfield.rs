use raylib::prelude::*;

pub struct TextField {
    pub text: String,
    cursor: usize,         // char index; 0 = before first char
    anchor: Option<usize>, // selection anchor; spans min(anchor,cursor)..max(anchor,cursor)
    filter: fn(char) -> bool,
}

impl TextField {
    pub fn new(filter: fn(char) -> bool) -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            anchor: None,
            filter,
        }
    }

    pub fn with_text(text: String, filter: fn(char) -> bool) -> Self {
        let cursor = text.chars().count();
        Self {
            text,
            cursor,
            anchor: None,
            filter,
        }
    }

    pub fn set_text(&mut self, text: String) {
        self.cursor = text.chars().count();
        self.text = text;
        self.anchor = None;
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
        self.anchor = None;
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }
    pub fn has_selection(&self) -> bool {
        self.anchor.is_some()
    }

    pub fn selection(&self) -> Option<(usize, usize)> {
        self.anchor.map(|a| (a.min(self.cursor), a.max(self.cursor)))
    }

    pub fn selected_text(&self) -> &str {
        self.selection()
            .map(|(lo, hi)| char_slice(&self.text, lo, hi))
            .unwrap_or("")
    }

    pub fn move_left(&mut self, shift: bool) {
        if !shift {
            if let Some((lo, _)) = self.selection() {
                self.cursor = lo;
                self.anchor = None;
                return;
            }
        }
        self.update_anchor(shift);
        if self.cursor > 0 {
            self.cursor -= 1;
        }
    }

    pub fn move_right(&mut self, shift: bool) {
        if !shift {
            if let Some((_, hi)) = self.selection() {
                self.cursor = hi;
                self.anchor = None;
                return;
            }
        }
        self.update_anchor(shift);
        if self.cursor < self.len_chars() {
            self.cursor += 1;
        }
    }

    pub fn move_home(&mut self, shift: bool) {
        self.update_anchor(shift);
        self.cursor = 0;
    }
    pub fn move_end(&mut self, shift: bool) {
        self.update_anchor(shift);
        self.cursor = self.len_chars();
    }

    /// Sets or clears the anchor depending on whether shift is held.
    fn update_anchor(&mut self, shift: bool) {
        if shift {
            self.anchor.get_or_insert(self.cursor);
        } else {
            self.anchor = None;
        }
    }

    pub fn insert_char(&mut self, c: char) {
        if !(self.filter)(c) {
            return;
        }
        self.delete_selection();
        let byte_pos = char_to_byte(&self.text, self.cursor);
        self.text.insert(byte_pos, c);
        self.cursor += 1;
    }

    pub fn insert_str(&mut self, s: &str) {
        s.chars().for_each(|c| self.insert_char(c));
    }

    pub fn backspace(&mut self) {
        if self.delete_selection() || self.cursor == 0 {
            return;
        }
        self.cursor -= 1;
        self.text.remove(char_to_byte(&self.text, self.cursor));
    }

    pub fn delete_forward(&mut self) {
        if self.delete_selection() || self.cursor >= self.len_chars() {
            return;
        }
        self.text.remove(char_to_byte(&self.text, self.cursor));
    }

    /// Deletes the selection if present; returns true if anything was deleted.
    fn delete_selection(&mut self) -> bool {
        let Some((lo, hi)) = self.selection() else { return false };
        self.text
            .drain(char_to_byte(&self.text, lo)..char_to_byte(&self.text, hi));
        self.cursor = lo;
        self.anchor = None;
        true
    }

    /// Processes all keyboard events for this frame. Call once per frame.
    pub fn handle_input(&mut self, rl: &mut RaylibHandle) {
        let shift = rl.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) || rl.is_key_down(KeyboardKey::KEY_RIGHT_SHIFT);

        self.handle_clipboard(rl);

        if key(rl, KeyboardKey::KEY_LEFT) {
            self.move_left(shift);
        }
        if key(rl, KeyboardKey::KEY_RIGHT) {
            self.move_right(shift);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_HOME) {
            self.move_home(shift);
        }
        if rl.is_key_pressed(KeyboardKey::KEY_END) {
            self.move_end(shift);
        }
        if key(rl, KeyboardKey::KEY_BACKSPACE) {
            self.backspace();
        }
        if key(rl, KeyboardKey::KEY_DELETE) {
            self.delete_forward();
        }

        while let Some(c) = rl.get_char_pressed() {
            if !c.is_control() {
                self.insert_char(c);
            }
        }
    }

    fn handle_clipboard(&mut self, rl: &mut RaylibHandle) {
        if !mod_down(rl) {
            return;
        }

        if rl.is_key_pressed(KeyboardKey::KEY_A) {
            self.anchor = Some(0);
            self.cursor = self.len_chars();
        } else if rl.is_key_pressed(KeyboardKey::KEY_C) {
            let _ = rl.set_clipboard_text(self.selected_text());
        } else if rl.is_key_pressed(KeyboardKey::KEY_X) {
            let _ = rl.set_clipboard_text(self.selected_text());
            self.delete_selection();
        } else if rl.is_key_pressed(KeyboardKey::KEY_V) {
            if let Ok(text) = rl.get_clipboard_text() {
                self.insert_str(&text);
            }
        }
    }

    /// Pixel offset from the field's text origin to the cursor.
    pub fn cursor_x(&self, fonts: &super::Fonts, size: f32) -> i32 {
        let s = char_slice(&self.text, 0, self.cursor);
        fonts.pick(s).measure_text(s, size, 1.0).x as i32
    }

    /// Pixel (x_offset, width) of the selection highlight, if any.
    pub fn selection_rect(&self, fonts: &super::Fonts, size: f32) -> Option<(i32, i32)> {
        let (lo, hi) = self.selection()?;
        let font = fonts.pick(&self.text);
        let x = font.measure_text(char_slice(&self.text, 0, lo), size, 1.0).x as i32;
        let w = font.measure_text(char_slice(&self.text, lo, hi), size, 1.0).x as i32;
        Some((x, w))
    }

    fn len_chars(&self) -> usize {
        self.text.chars().count()
    }
}

fn mod_down(rl: &RaylibHandle) -> bool {
    rl.is_key_down(KeyboardKey::KEY_LEFT_CONTROL)
        || rl.is_key_down(KeyboardKey::KEY_RIGHT_CONTROL)
        || rl.is_key_down(KeyboardKey::KEY_LEFT_SUPER)
        || rl.is_key_down(KeyboardKey::KEY_RIGHT_SUPER)
}

/// Returns true if a key was pressed or is being held (repeat).
fn key(rl: &RaylibHandle, k: KeyboardKey) -> bool {
    rl.is_key_pressed(k) || rl.is_key_pressed_repeat(k)
}

fn char_to_byte(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map(|(b, _)| b).unwrap_or(s.len())
}

fn char_slice(s: &str, lo: usize, hi: usize) -> &str {
    &s[char_to_byte(s, lo)..char_to_byte(s, hi)]
}
