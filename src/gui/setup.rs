use std::path::Path;

use raylib::prelude::*;

use crate::toml::parser::{Config, General, Members};

use super::WIN_W;

const FONT_SIZE: i32 = 20;
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
            Some(c) => {
                let member_count = c.members.students.len();
                Self {
                    step: Step::GroupName,
                    group_name: c.general.group_name.clone(),
                    my_name: c.general.my_name.clone(),
                    member_count,
                    members: c.members.students.clone(),
                    input: c.general.group_name.clone(),
                    error: None,
                }
            }
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
                    if n == 1 {
                        self.step = Step::Confirm;
                    } else {
                        self.step = Step::MemberName(1);
                    }
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
                    if val.trim().eq_ignore_ascii_case(&self.my_name) {
                        self.error = Some("Your name has already been added.".into());
                    } else {
                        self.error = Some(format!("\"{}\" has already been added.", val));
                    }
                    return;
                }
                self.members.push(val);
                self.input = String::new();
                if idx + 1 < self.member_count {
                    self.step = Step::MemberName(idx + 1);
                } else {
                    self.step = Step::Confirm;
                }
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
    config_path: &Path,
    prefill: Option<&Config>,
) -> Config {
    rl.set_window_size(WIN_W, WIN_H);

    let mut state = SetupState::new(prefill);

    loop {
        // ── Input ─────────────────────────────────────────────────────────────

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

        // ── Draw ──────────────────────────────────────────────────────────────

        let mut d = rl.begin_drawing(thread);
        d.clear_background(BG);

        match &state.step {
            Step::Confirm => draw_confirm(&mut d, &state),
            _ => draw_input(&mut d, &state),
        }
    }
}

fn draw_input(d: &mut RaylibDrawHandle, state: &SetupState) {
    let (title, sub) = step_labels(&state.step, state.member_count);

    d.draw_text(title, PAD, PAD, FONT_SIZE + 2, FG);
    d.draw_text(sub, PAD, PAD + 32, FONT_SIZE - 2, DIM);

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

    let display = if state.input.is_empty() { DIM } else { FG };
    let text = if state.input.is_empty() {
        "type and press Enter"
    } else {
        &state.input
    };
    d.draw_text(
        text,
        bx + 8,
        by + (INPUT_H - FONT_SIZE) / 2,
        FONT_SIZE,
        display,
    );

    let cx = bx + 8 + d.measure_text(&state.input, FONT_SIZE) + 1;
    if (d.get_time() * 2.0) as i32 % 2 == 0 {
        d.draw_text("|", cx, by + (INPUT_H - FONT_SIZE) / 2, FONT_SIZE, ACCENT);
    }

    if let Some(err) = &state.error {
        d.draw_text(err, PAD, by + INPUT_H + 12, FONT_SIZE - 2, RED);
    }

    d.draw_text(
        "Press Enter to continue",
        PAD,
        WIN_H - PAD - FONT_SIZE,
        FONT_SIZE - 4,
        DIM,
    );
}

fn draw_confirm(d: &mut RaylibDrawHandle, state: &SetupState) {
    d.draw_text("Confirm config", PAD, PAD, FONT_SIZE + 2, FG);

    let mut y = PAD + 36;
    let line_h = FONT_SIZE + 6;

    d.draw_text(
        &format!("Group:  {}", state.group_name),
        PAD,
        y,
        FONT_SIZE - 2,
        DIM,
    );
    y += line_h;
    d.draw_text(
        &format!("You:    {}", state.my_name),
        PAD,
        y,
        FONT_SIZE - 2,
        DIM,
    );
    y += line_h;
    d.draw_text("Members:", PAD, y, FONT_SIZE - 2, DIM);
    y += line_h;

    for (i, name) in state.members.iter().enumerate() {
        let label = if name.trim().eq_ignore_ascii_case(&state.my_name) {
            format!("  {}. {} (you)", i + 1, name)
        } else {
            format!("  {}. {}", i + 1, name)
        };
        d.draw_text(&label, PAD, y, FONT_SIZE - 2, FG);
        y += line_h;
    }

    y += 6;
    d.draw_text(
        "[Y] Save and continue   [N] Start over",
        PAD,
        y,
        FONT_SIZE - 2,
        GREEN,
    );
}

fn step_labels(step: &Step, total: usize) -> (&'static str, &'static str) {
    match step {
        Step::GroupName => ("Setup — Group name", "Enter your group's name"),
        Step::MyName => (
            "Setup — Your name",
            "Enter your full name as registered at LNU",
        ),
        Step::MemberCount => ("Setup — Team size", "How many members in total? (1–7)"),
        Step::MemberName(i) => {
            let _ = (i, total);
            ("Setup — Member name", "Enter this member's full name")
        }
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
