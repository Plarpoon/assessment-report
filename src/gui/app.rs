use raylib::prelude::*;

use crate::toml::parser::Config;
use crate::veuros::{Assignment, TOTAL};

use super::clipboard::{self, ClipboardAction};
use super::{Fonts, WIN_W};

const FONT_SIZE: f32 = 20.0;
const PAD: i32 = 24;
const ROW_H: i32 = 48;
const BOX_W: i32 = 90;
const BOX_H: i32 = 32;
const BTN_W: i32 = 90;
const BTN_H: i32 = 22;
const SOLO_H: i32 = 160;

#[rustfmt::skip]
mod colors {
    use raylib::prelude::Color;
    pub const BG:      Color = Color { r: 24,  g: 24,  b: 32,  a: 255 };
    pub const FG:      Color = Color { r: 220, g: 220, b: 220, a: 255 };
    pub const ACCENT:  Color = Color { r: 90,  g: 160, b: 255, a: 255 };
    pub const RED:     Color = Color { r: 220, g: 60,  b: 60,  a: 255 };
    pub const BOX_BG:  Color = Color { r: 36,  g: 36,  b: 48,  a: 255 };
    pub const BOX_ACT: Color = Color { r: 50,  g: 50,  b: 70,  a: 255 };
    pub const GREEN:   Color = Color { r: 80,  g: 200, b: 120, a: 255 };
    pub const DIM:     Color = Color { r: 100, g: 100, b: 120, a: 255 };
    pub const BTN_BG:  Color = Color { r: 40,  g: 40,  b: 55,  a: 255 };
    pub const SEL_BG:  Color = Color { r: 90,  g: 160, b: 255, a: 80  };
}
use colors::*;

pub enum Outcome {
    Done(Vec<Assignment>),
    EditConfig,
}

#[derive(PartialEq)]
enum Screen {
    Assign,
    Confirm,
    Solo,
}

struct State<'a> {
    peers: Vec<&'a str>,
    buffers: Vec<String>,
    active: usize,
    selected_all: bool,
    screen: Screen,
}

impl<'a> State<'a> {
    fn new(peers: Vec<&'a str>) -> Self {
        let n = peers.len();
        Self {
            peers,
            buffers: vec![String::new(); n],
            active: 0,
            selected_all: false,
            screen: if n == 0 { Screen::Solo } else { Screen::Assign },
        }
    }

    fn remaining(&self) -> i64 {
        let total: u32 = self.buffers.iter().map(|b| b.parse::<u32>().unwrap_or(0)).sum();
        TOTAL as i64 - total as i64
    }

    fn assignments(&self) -> Vec<Assignment> {
        self.peers
            .iter()
            .zip(&self.buffers)
            .map(|(&name, buf)| Assignment {
                name: name.to_string(),
                amount: buf.parse().unwrap_or(0),
            })
            .collect()
    }

    fn win_h(&self) -> i32 {
        match self.screen {
            Screen::Solo => SOLO_H,
            _ => PAD * 3 + ROW_H * self.peers.len() as i32 + PAD * 2 + BOX_H + PAD,
        }
    }
}

pub fn run(rl: &mut RaylibHandle, thread: &RaylibThread, fonts: &Fonts, config: &Config) -> Outcome {
    let my_name = config.general.my_name.trim();

    let peers: Vec<&str> = config
        .members
        .students
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.eq_ignore_ascii_case(my_name))
        .collect();

    let mut state = State::new(peers);
    rl.set_window_size(WIN_W, state.win_h());

    loop {
        if rl.window_should_close() {
            std::process::exit(0);
        }

        if let Some(outcome) = handle_input(rl, &mut state) {
            return outcome;
        }

        let mut d = rl.begin_drawing(thread);
        d.clear_background(BG);

        match state.screen {
            Screen::Solo => draw_solo(&mut d, fonts, state.win_h()),
            Screen::Confirm => draw_confirm(&mut d, fonts, state.win_h()),
            Screen::Assign => draw_assign(&mut d, fonts, &state),
        }
    }
}

fn handle_input(rl: &mut RaylibHandle, state: &mut State) -> Option<Outcome> {
    match state.screen {
        Screen::Solo => handle_solo(rl),
        Screen::Confirm => handle_confirm(rl, state),
        Screen::Assign => handle_assign(rl, state),
    }
}

fn handle_solo(rl: &mut RaylibHandle) -> Option<Outcome> {
    if rl.is_key_pressed(KeyboardKey::KEY_Y) {
        return Some(Outcome::Done(vec![]));
    }
    if rl.is_key_pressed(KeyboardKey::KEY_N) {
        return Some(Outcome::EditConfig);
    }
    None
}

fn handle_confirm(rl: &mut RaylibHandle, state: &mut State) -> Option<Outcome> {
    if rl.is_key_pressed(KeyboardKey::KEY_Y) {
        return Some(Outcome::Done(state.assignments()));
    }
    if rl.is_key_pressed(KeyboardKey::KEY_N) {
        state.screen = Screen::Assign;
    }
    None
}

fn handle_assign(rl: &mut RaylibHandle, state: &mut State) -> Option<Outcome> {
    let win_h = state.win_h();

    if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
        let pos = rl.get_mouse_position();
        if hit(pos, edit_btn_rect(win_h)) {
            return Some(Outcome::EditConfig);
        }
        if let Some(i) = (0..state.peers.len()).find(|&i| hit(pos, field_rect(i))) {
            if state.active != i {
                state.selected_all = false;
            }
            state.active = i;
        }
    }

    if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
        state.active = (state.active + 1) % state.peers.len();
        state.selected_all = false;
    }

    handle_digit_input(rl, state);

    if rl.is_key_pressed(KeyboardKey::KEY_ENTER) && state.remaining() == 0 {
        state.screen = Screen::Confirm;
    }

    None
}

fn handle_digit_input(rl: &mut RaylibHandle, state: &mut State) {
    let buf = &mut state.buffers[state.active];

    // A digit while select-all is active replaces the whole buffer.
    while let Some(c) = rl.get_char_pressed() {
        if c.is_ascii_digit() {
            if state.selected_all {
                buf.clear();
            }
            buf.push(c);
            state.selected_all = false;
        }
    }

    match clipboard::handle(rl, buf, |c| c.is_ascii_digit()) {
        ClipboardAction::Replace(s) => {
            *buf = s;
            state.selected_all = false;
        }
        ClipboardAction::SelectAll => {
            state.selected_all = true;
        }
        ClipboardAction::Noop => {}
    }

    if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) || rl.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE) {
        if state.selected_all {
            buf.clear();
        } else {
            buf.pop();
        }
        state.selected_all = false;
    }
}

fn draw_solo(d: &mut RaylibDrawHandle, fonts: &Fonts, win_h: i32) {
    let msg = "You are the only member.";
    let sub = "Write file with only ME, -?  [Y] yes   [N] edit config";
    txt(
        d,
        fonts,
        msg,
        (WIN_W - measure(fonts, msg, FONT_SIZE + 2.0)) / 2,
        win_h / 2 - 24,
        FONT_SIZE + 2.0,
        FG,
    );
    txt(
        d,
        fonts,
        sub,
        (WIN_W - measure(fonts, sub, FONT_SIZE - 2.0)) / 2,
        win_h / 2 + 12,
        FONT_SIZE - 2.0,
        DIM,
    );
}

fn draw_assign(d: &mut RaylibDrawHandle, fonts: &Fonts, state: &State) {
    let win_h = state.win_h();
    let remaining = state.remaining();

    txt(d, fonts, "Assign 50 vEuros", PAD, PAD, FONT_SIZE, FG);

    let rem_color = match remaining.cmp(&0) {
        std::cmp::Ordering::Less => RED,
        std::cmp::Ordering::Equal => GREEN,
        std::cmp::Ordering::Greater => ACCENT,
    };
    let rem_text = format!("Remaining: {remaining}");
    let tw = measure(fonts, &rem_text, FONT_SIZE);
    txt(d, fonts, &rem_text, WIN_W - PAD - tw, PAD, FONT_SIZE, rem_color);

    for (i, &name) in state.peers.iter().enumerate() {
        draw_row(d, fonts, state, i, name);
    }

    let hint = match remaining.cmp(&0) {
        std::cmp::Ordering::Equal => "Press Enter to confirm",
        std::cmp::Ordering::Less => "Over budget — reduce some values",
        std::cmp::Ordering::Greater => "Click or Tab to select a field",
    };
    txt(
        d,
        fonts,
        hint,
        PAD,
        PAD * 3 + state.peers.len() as i32 * ROW_H + PAD,
        FONT_SIZE - 2.0,
        DIM,
    );

    draw_edit_btn(d, fonts, win_h);
}

fn draw_row(d: &mut RaylibDrawHandle, fonts: &Fonts, state: &State, i: usize, name: &str) {
    let y = PAD * 3 + i as i32 * ROW_H;
    let is_active = state.active == i;
    let r = field_rect(i);
    let val = &state.buffers[i];
    let tx = r.x as i32 + 8;
    let ty = r.y as i32 + (BOX_H - FONT_SIZE as i32) / 2;

    txt(d, fonts, name, PAD, y + (BOX_H - FONT_SIZE as i32) / 2, FONT_SIZE, FG);

    d.draw_rectangle_rec(r, if is_active { BOX_ACT } else { BOX_BG });
    d.draw_rectangle_lines_ex(r, 1.5, if is_active { ACCENT } else { DIM });

    // Show the placeholder "0" only when unfocused and empty.
    let (val_text, val_color) = if val.is_empty() && !is_active {
        ("0", DIM)
    } else {
        (val.as_str(), FG)
    };

    // Draw selection highlight behind the text when select-all is active.
    if is_active && state.selected_all && !val.is_empty() {
        d.draw_rectangle(tx, ty, measure(fonts, val, FONT_SIZE), FONT_SIZE as i32, SEL_BG);
    }

    txt(d, fonts, val_text, tx, ty, FONT_SIZE, val_color);

    if is_active && !state.selected_all && (d.get_time() * 2.0) as i32 % 2 == 0 {
        txt(
            d,
            fonts,
            "|",
            tx + measure(fonts, val, FONT_SIZE),
            ty,
            FONT_SIZE,
            ACCENT,
        );
    }
}

fn draw_confirm(d: &mut RaylibDrawHandle, fonts: &Fonts, win_h: i32) {
    let msg = "All 50 vEuros assigned.";
    let sub = "Confirm and write file?  [Y] yes   [N] back";
    txt(
        d,
        fonts,
        msg,
        (WIN_W - measure(fonts, msg, FONT_SIZE + 4.0)) / 2,
        win_h / 2 - 30,
        FONT_SIZE + 4.0,
        Color::WHITE,
    );
    txt(
        d,
        fonts,
        sub,
        (WIN_W - measure(fonts, sub, FONT_SIZE)) / 2,
        win_h / 2 + 10,
        FONT_SIZE,
        DIM,
    );
}

fn draw_edit_btn(d: &mut RaylibDrawHandle, fonts: &Fonts, win_h: i32) {
    let r = edit_btn_rect(win_h);
    let lw = measure(fonts, "Edit config", FONT_SIZE - 6.0);
    d.draw_rectangle_rec(r, BTN_BG);
    d.draw_rectangle_lines_ex(r, 1.0, DIM);
    txt(
        d,
        fonts,
        "Edit config",
        r.x as i32 + (BTN_W - lw) / 2,
        r.y as i32 + (BTN_H - (FONT_SIZE as i32 - 6)) / 2,
        FONT_SIZE - 6.0,
        DIM,
    );
}

fn txt(d: &mut RaylibDrawHandle, fonts: &Fonts, text: &str, x: i32, y: i32, size: f32, color: Color) {
    d.draw_text_ex(
        fonts.pick(text),
        text,
        Vector2 {
            x: x as f32,
            y: y as f32,
        },
        size,
        1.0,
        color,
    );
}

fn measure(fonts: &Fonts, text: &str, size: f32) -> i32 {
    fonts.pick(text).measure_text(text, size, 1.0).x as i32
}

fn hit(pos: Vector2, r: Rectangle) -> bool {
    pos.x >= r.x && pos.x <= r.x + r.width && pos.y >= r.y && pos.y <= r.y + r.height
}

fn field_rect(row: usize) -> Rectangle {
    Rectangle {
        x: (WIN_W - PAD - BOX_W) as f32,
        y: (PAD * 3 + row as i32 * ROW_H) as f32,
        width: BOX_W as f32,
        height: BOX_H as f32,
    }
}

fn edit_btn_rect(win_h: i32) -> Rectangle {
    Rectangle {
        x: (WIN_W - PAD - BTN_W) as f32,
        y: (win_h - PAD - BTN_H) as f32,
        width: BTN_W as f32,
        height: BTN_H as f32,
    }
}
