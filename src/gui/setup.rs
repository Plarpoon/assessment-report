use std::path::Path;

use raylib::prelude::*;

use crate::toml::parser::{Config, General, Members};

use super::WIN_W;

const FONT_SIZE: f32 = 20.0;
const PAD: i32 = 24;
const INPUT_W: i32 = 360;
const INPUT_H: i32 = 36;
const WIN_H: i32 = 260;

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
const BOX_ACT: Color = Color {
    r: 50,
    g: 50,
    b: 70,
    a: 255,
};
const DIM: Color = Color {
    r: 100,
    g: 100,
    b: 120,
    a: 255,
};
const RED: Color = Color {
    r: 220,
    g: 60,
    b: 60,
    a: 255,
};
const GREEN: Color = Color {
    r: 80,
    g: 200,
    b: 120,
    a: 255,
};

#[derive(PartialEq)]
enum Step {
    GroupName,
    MyName,
    MemberCount,
    MemberName(usize),
    Confirm,
}

struct SetupState {
    step: Step,
    group_name: String,
    my_name: String,
    member_count: usize,
    members: Vec<String>,
    input: String,
    error: Option<String>,
}

impl SetupState {
    fn new(prefill: Option<&Config>) -> Self {
        match prefill {
            None => Self {
                step: Step::GroupName,
                group_name: String::new(),
                my_name: String::new(),
                member_count: 0,
                members: Vec::new(),
                input: String::new(),
                error: None,
            },
            Some(c) => Self {
                step: Step::GroupName,
                group_name: c.general.group_name.clone(),
                my_name: c.general.my_name.clone(),
                member_count: c.members.students.len(),
                members: c.members.students.clone(),
                input: c.general.group_name.clone(),
                error: None,
            },
        }
    }

    fn advance(&mut self) {
        self.error = None;
        match &self.step {
            Step::GroupName => {
                let val = self.input.trim().to_string();
                if val.is_empty() {
                    self.error = Some("Group name cannot be empty.".into());
                    return;
                }
                self.group_name = val;
                self.input = self.my_name.clone();
                self.step = Step::MyName;
            }
            Step::MyName => {
                let val = self.input.trim().to_string();
                if val.is_empty() {
                    self.error = Some("Your name cannot be empty.".into());
                    return;
                }
                self.my_name = val.clone();
                self.members = vec![val];
                self.input = String::new();
                self.step = Step::MemberCount;
            }
            Step::MemberCount => match self.input.trim().parse::<usize>() {
                Ok(n) if (1..=7).contains(&n) => {
                    self.member_count = n;
                    self.input = String::new();
                    self.step = if n == 1 {
                        Step::Confirm
                    } else {
                        Step::MemberName(1)
                    };
                }
                Ok(_) => self.error = Some("Enter a number between 1 and 7.".into()),
                Err(_) => self.error = Some("Please enter a number.".into()),
            },
            Step::MemberName(idx) => {
                let idx = *idx;
                let val = self.input.trim().to_string();
                if val.is_empty() {
                    self.error = Some("Name cannot be empty.".into());
                    return;
                }
                if self
                    .members
                    .iter()
                    .any(|m| m.trim().eq_ignore_ascii_case(&val))
                {
                    self.error = Some(if val.trim().eq_ignore_ascii_case(&self.my_name) {
                        "Your name has already been added.".into()
                    } else {
                        format!("\"{}\" has already been added.", val)
                    });
                    return;
                }
                self.members.push(val);
                self.input = String::new();
                self.step = if idx + 1 < self.member_count {
                    Step::MemberName(idx + 1)
                } else {
                    Step::Confirm
                };
            }
            Step::Confirm => {}
        }
    }

    fn to_config(&self) -> Config {
        Config {
            general: General {
                group_name: self.group_name.clone(),
                my_name: self.my_name.clone(),
            },
            members: Members {
                students: self.members.clone(),
            },
        }
    }
}

pub fn run(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    font: &Font,
    config_path: &Path,
    prefill: Option<&Config>,
) -> Config {
    rl.set_window_size(WIN_W, WIN_H);

    let mut state = SetupState::new(prefill);

    loop {
        if state.step == Step::Confirm {
            if rl.is_key_pressed(KeyboardKey::KEY_Y) {
                let config = state.to_config();
                write_config(config_path, &config);
                return config;
            }
            if rl.is_key_pressed(KeyboardKey::KEY_N) {
                state = SetupState::new(prefill);
            }
        } else {
            while let Some(c) = rl.get_char_pressed() {
                if !c.is_control() {
                    state.input.push(c);
                }
            }
            if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE)
                || rl.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE)
            {
                state.input.pop();
            }
            if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                state.advance();
            }
        }

        if rl.window_should_close() {
            std::process::exit(0);
        }

        let mut d = rl.begin_drawing(thread);
        d.clear_background(BG);

        match &state.step {
            Step::Confirm => draw_confirm(&mut d, font, &state),
            _ => draw_input(&mut d, font, &state),
        }
    }
}

fn txt(d: &mut RaylibDrawHandle, font: &Font, text: &str, x: i32, y: i32, size: f32, color: Color) {
    d.draw_text_ex(
        font,
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

fn measure(font: &Font, text: &str, size: f32) -> i32 {
    font.measure_text(text, size, 1.0).x as i32
}

fn draw_input(d: &mut RaylibDrawHandle, font: &Font, state: &SetupState) {
    let (title, sub) = step_labels(&state.step, state.member_count);

    txt(d, font, title, PAD, PAD, FONT_SIZE + 2.0, FG);
    txt(d, font, sub, PAD, PAD + 32, FONT_SIZE - 2.0, DIM);

    let bx = PAD;
    let by = PAD + 70;
    let rect = Rectangle {
        x: bx as f32,
        y: by as f32,
        width: INPUT_W as f32,
        height: INPUT_H as f32,
    };
    d.draw_rectangle_rec(rect, BOX_ACT);
    d.draw_rectangle_lines_ex(rect, 1.5, ACCENT);

    let (display_color, text) = if state.input.is_empty() {
        (DIM, "type and press Enter")
    } else {
        (FG, state.input.as_str())
    };
    txt(
        d,
        font,
        text,
        bx + 8,
        by + (INPUT_H - FONT_SIZE as i32) / 2,
        FONT_SIZE,
        display_color,
    );

    if (d.get_time() * 2.0) as i32 % 2 == 0 {
        let cx = bx + 8 + measure(font, &state.input, FONT_SIZE) + 1;
        txt(
            d,
            font,
            "|",
            cx,
            by + (INPUT_H - FONT_SIZE as i32) / 2,
            FONT_SIZE,
            ACCENT,
        );
    }

    if let Some(err) = &state.error {
        txt(d, font, err, PAD, by + INPUT_H + 12, FONT_SIZE - 2.0, RED);
    }

    txt(
        d,
        font,
        "Press Enter to continue",
        PAD,
        WIN_H - PAD - FONT_SIZE as i32,
        FONT_SIZE - 4.0,
        DIM,
    );
}

fn draw_confirm(d: &mut RaylibDrawHandle, font: &Font, state: &SetupState) {
    txt(d, font, "Confirm config", PAD, PAD, FONT_SIZE + 2.0, FG);

    let mut y = PAD + 36;
    let line_h = FONT_SIZE as i32 + 6;

    txt(
        d,
        font,
        &format!("Group:  {}", state.group_name),
        PAD,
        y,
        FONT_SIZE - 2.0,
        DIM,
    );
    y += line_h;
    txt(
        d,
        font,
        &format!("You:    {}", state.my_name),
        PAD,
        y,
        FONT_SIZE - 2.0,
        DIM,
    );
    y += line_h;
    txt(d, font, "Members:", PAD, y, FONT_SIZE - 2.0, DIM);
    y += line_h;

    for (i, name) in state.members.iter().enumerate() {
        let label = if name.trim().eq_ignore_ascii_case(&state.my_name) {
            format!("  {}. {} (you)", i + 1, name)
        } else {
            format!("  {}. {}", i + 1, name)
        };
        txt(d, font, &label, PAD, y, FONT_SIZE - 2.0, FG);
        y += line_h;
    }

    y += 6;
    txt(
        d,
        font,
        "[Y] Save and continue   [N] Start over",
        PAD,
        y,
        FONT_SIZE - 2.0,
        GREEN,
    );
}

fn step_labels(step: &Step, _total: usize) -> (&'static str, &'static str) {
    match step {
        Step::GroupName => ("Setup — Group name", "Enter your group's name"),
        Step::MyName => (
            "Setup — Your name",
            "Enter your full name as registered at LNU",
        ),
        Step::MemberCount => ("Setup — Team size", "How many members in total? (1–7)"),
        Step::MemberName(_) => ("Setup — Member name", "Enter this member's full name"),
        Step::Confirm => ("", ""),
    }
}

fn write_config(path: &Path, config: &Config) {
    let content = format!(
        "[general]\ngroup_name = \"{}\"\nmy_name    = \"{}\"\n\n[members]\nstudents = [\n{}\n]\n",
        config.general.group_name,
        config.general.my_name,
        config
            .members
            .students
            .iter()
            .map(|s| format!("    \"{}\",", s))
            .collect::<Vec<_>>()
            .join("\n")
    );
    std::fs::write(path, content)
        .unwrap_or_else(|e| eprintln!("Warning: could not write config: {e}"));
}
