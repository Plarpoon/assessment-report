use raylib::prelude::*;

use crate::toml::parser::Config;
use crate::veuros::{Assignment, TOTAL};

const WIN_W: i32 = 520;
const FONT_SIZE: i32 = 20;
const PAD: i32 = 24;
const ROW_H: i32 = 48;
const BOX_W: i32 = 90;
const BOX_H: i32 = 32;

const BG: Color = Color {
    r: 24,
    g: 24,
    b: 32,
    a: 255,
};
const FG: Color = Color {
    r: 220,
    g: 220,
    b: 220,
    a: 255,
};
const ACCENT: Color = Color {
    r: 90,
    g: 160,
    b: 255,
    a: 255,
};
const RED: Color = Color {
    r: 220,
    g: 60,
    b: 60,
    a: 255,
};
const BOX_BG: Color = Color {
    r: 36,
    g: 36,
    b: 48,
    a: 255,
};
const BOX_ACT: Color = Color {
    r: 50,
    g: 50,
    b: 70,
    a: 255,
};
const GREEN: Color = Color {
    r: 80,
    g: 200,
    b: 120,
    a: 255,
};
const DIM: Color = Color {
    r: 100,
    g: 100,
    b: 120,
    a: 255,
};

#[derive(PartialEq)]
enum Screen {
    Assign,
    Confirm,
    Done,
}

struct State<'a> {
    peers: Vec<&'a str>,
    buffers: Vec<String>,
    active: usize,
    screen: Screen,
}

impl<'a> State<'a> {
    fn new(peers: Vec<&'a str>) -> Self {
        let len = peers.len();
        Self {
            peers,
            buffers: vec![String::new(); len],
            active: 0,
            screen: Screen::Assign,
        }
    }

    fn amounts(&self) -> Vec<u32> {
        self.buffers
            .iter()
            .map(|b| b.parse::<u32>().unwrap_or(0))
            .collect()
    }

    fn total(&self) -> u32 {
        self.amounts().iter().sum()
    }

    fn remaining(&self) -> i64 {
        TOTAL as i64 - self.total() as i64
    }

    fn assignments(&self) -> Vec<Assignment> {
        self.peers
            .iter()
            .zip(self.amounts())
            .map(|(&name, amount)| Assignment {
                name: name.to_string(),
                amount,
            })
            .collect()
    }
}

pub fn run(config: &Config) -> Vec<Assignment> {
    let my_name = config.general.my_name.trim();

    let peers: Vec<&str> = config
        .members
        .students
        .iter()
        .map(|s| s.trim())
        .filter(|s| !s.eq_ignore_ascii_case(my_name))
        .collect();

    let win_h = PAD * 3 + ROW_H * peers.len() as i32 + PAD * 2 + BOX_H + PAD;
    let mut state = State::new(peers);

    let (mut rl, thread) = raylib::init()
        .size(WIN_W, win_h)
        .title("vEuro Assignment")
        .build();

    rl.set_target_fps(60);

    loop {
        if state.screen == Screen::Assign {
            // Click to focus a field
            if rl.is_mouse_button_pressed(MouseButton::MOUSE_BUTTON_LEFT) {
                let pos = rl.get_mouse_position();
                for (i, _) in state.peers.iter().enumerate() {
                    let r = field_rect(i);
                    if pos.x >= r.x
                        && pos.x <= r.x + r.width
                        && pos.y >= r.y
                        && pos.y <= r.y + r.height
                    {
                        state.active = i;
                    }
                }
            }

            // Tab cycles focus
            if rl.is_key_pressed(KeyboardKey::KEY_TAB) {
                state.active = (state.active + 1) % state.peers.len();
            }

            // Character input — digits only
            while let Some(c) = rl.get_char_pressed() {
                if c.is_ascii_digit() {
                    state.buffers[state.active].push(c);
                }
            }

            // Backspace
            if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                || rl.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE)
            {
                state.buffers[state.active].pop();
            }

            // Enter when all 50 assigned → go to confirm
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) && state.remaining() == 0 {
                state.screen = Screen::Confirm;
            }
        } else if state.screen == Screen::Confirm {
            if rl.is_key_pressed(KeyboardKey::KEY_Y) {
                state.screen = Screen::Done;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_N) {
                state.screen = Screen::Assign;
            }
        }

        if state.screen == Screen::Done {
            return state.assignments();
        }

        if rl.window_should_close() {
            std::process::exit(0);
        }


        
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(BG);

        if state.screen == Screen::Confirm {
            draw_confirm(&mut d, win_h);
            continue;
        }

        // Header
        d.draw_text("Assign 50 vEuros", PAD, PAD, FONT_SIZE, FG);

        let remaining = state.remaining();
        let rem_color = if remaining < 0 {
            RED
        } else if remaining == 0 {
            GREEN
        } else {
            ACCENT
        };
        let rem_text = format!("Remaining: {remaining}");
        let tw = d.measure_text(&rem_text, FONT_SIZE);
        d.draw_text(&rem_text, WIN_W - PAD - tw, PAD, FONT_SIZE, rem_color);

        // Rows
        for (i, &name) in state.peers.iter().enumerate() {
            let y = PAD * 3 + i as i32 * ROW_H;
            let is_active = state.active == i;

            d.draw_text(name, PAD, y + (BOX_H - FONT_SIZE) / 2, FONT_SIZE, FG);

            let r = field_rect(i);
            d.draw_rectangle_rec(r, if is_active { BOX_ACT } else { BOX_BG });
            d.draw_rectangle_lines_ex(r, 1.5, if is_active { ACCENT } else { DIM });

            let val = &state.buffers[i];
            let tx = r.x as i32 + 8;
            let ty = r.y as i32 + (BOX_H - FONT_SIZE) / 2;
            if val.is_empty() {
                d.draw_text("0", tx, ty, FONT_SIZE, DIM);
            } else {
                d.draw_text(val, tx, ty, FONT_SIZE, FG);
            }

            // Cursor blink
            if is_active && (d.get_time() * 2.0) as i32 % 2 == 0 {
                let cx = tx + d.measure_text(val, FONT_SIZE);
                d.draw_text("|", cx, ty, FONT_SIZE, ACCENT);
            }
        }

        // Hint at bottom
        let hint_y = PAD * 3 + state.peers.len() as i32 * ROW_H + PAD;
        let hint = if remaining == 0 {
            "Press Enter to confirm"
        } else if remaining < 0 {
            "Over budget — reduce some values"
        } else {
            "Click or Tab to select a field"
        };
        d.draw_text(hint, PAD, hint_y, FONT_SIZE - 2, DIM);
    }
}

fn field_rect(row: usize) -> Rectangle {
    Rectangle {
        x: (WIN_W - PAD - BOX_W) as f32,
        y: (PAD * 3 + row as i32 * ROW_H) as f32,
        width: BOX_W as f32,
        height: BOX_H as f32,
    }
}

fn draw_confirm(d: &mut RaylibDrawHandle, win_h: i32) {
    let msg = "All 50 vEuros assigned.";
    let sub = "Confirm and write file?  [Y] yes   [N] back";
    let mw = d.measure_text(msg, FONT_SIZE + 4);
    let sw = d.measure_text(sub, FONT_SIZE);
    d.draw_text(
        msg,
        (WIN_W - mw) / 2,
        win_h / 2 - 30,
        FONT_SIZE + 4,
        Color::WHITE,
    );
    d.draw_text(sub, (WIN_W - sw) / 2, win_h / 2 + 10, FONT_SIZE, DIM);
}
