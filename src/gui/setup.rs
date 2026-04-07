use std::path::Path;

use raylib::prelude::*;

use crate::toml::parser::{Config, General, Members};

use super::clipboard::ClipboardAction;
use super::{Fonts, WIN_W};

const FONT_SIZE: f32 = 20.0;
const PAD: i32 = 24;
const INPUT_W: i32 = 360;
const INPUT_H: i32 = 36;

// Height for input steps (fixed — only one field visible at a time).
const INPUT_WIN_H: i32 = 260;

#[rustfmt::skip]
mod colors {
    use raylib::prelude::Color;
    pub const BG:      Color = Color { r: 24,  g: 24,  b: 32,  a: 255 };
    pub const FG:      Color = Color { r: 220, g: 220, b: 220, a: 255 };
    pub const ACCENT:  Color = Color { r: 90,  g: 160, b: 255, a: 255 };
    pub const BOX_ACT: Color = Color { r: 50,  g: 50,  b: 70,  a: 255 };
    pub const DIM:     Color = Color { r: 100, g: 100, b: 120, a: 255 };
    pub const RED:     Color = Color { r: 220, g: 60,  b: 60,  a: 255 };
    pub const GREEN:   Color = Color { r: 80,  g: 200, b: 120, a: 255 };
    pub const SEL_BG:  Color = Color { r: 90,  g: 160, b: 255, a: 80  };
}
use colors::*;

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
    selected_all: bool,
    error: Option<String>,
}

impl Default for SetupState {
    fn default() -> Self {
        Self {
            step: Step::GroupName,
            group_name: String::new(),
            my_name: String::new(),
            member_count: 0,
            members: Vec::new(),
            input: String::new(),
            selected_all: false,
            error: None,
        }
    }
}

impl SetupState {
    fn from_config(c: &Config) -> Self {
        Self {
            group_name: c.general.group_name.clone(),
            my_name: c.general.my_name.clone(),
            member_count: c.members.students.len(),
            members: c.members.students.clone(),
            input: c.general.group_name.clone(),
            ..Self::default()
        }
    }

    fn advance(&mut self) {
        self.error = None;
        self.selected_all = false;
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
                    self.step = if n == 1 { Step::Confirm } else { Step::MemberName(1) };
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
                let duplicate = self.members.iter().any(|m| m.trim().eq_ignore_ascii_case(&val));
                if duplicate {
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

    /// Window height for the confirm screen, scaled to member count.
    fn confirm_win_h(&self) -> i32 {
        let line_h = FONT_SIZE as i32 + 6;
        let n_lines = 4 + self.members.len() as i32; // title + group + you + members header + N names + action
        PAD + line_h * n_lines + PAD * 2
    }
}

pub fn run(
    rl: &mut RaylibHandle,
    thread: &RaylibThread,
    fonts: &Fonts,
    config_path: &Path,
    prefill: Option<&Config>,
) -> Config {
    rl.set_window_size(WIN_W, INPUT_WIN_H);

    let mut state = prefill.map(SetupState::from_config).unwrap_or_default();

    loop {
        match state.step {
            Step::Confirm => {
                // Resize window to fit however many members we have.
                rl.set_window_size(WIN_W, state.confirm_win_h());

                if rl.is_key_pressed(KeyboardKey::KEY_Y) {
                    let config = state.to_config();
                    if let Err(e) = write_config(config_path, &config) {
                        eprintln!("Error: could not write config: {e}");
                        std::process::exit(1);
                    }
                    return config;
                }
                if rl.is_key_pressed(KeyboardKey::KEY_N) {
                    rl.set_window_size(WIN_W, INPUT_WIN_H);
                    state = prefill.map(SetupState::from_config).unwrap_or_default();
                }
            }
            _ => {
                rl.set_window_size(WIN_W, INPUT_WIN_H);

                while let Some(c) = rl.get_char_pressed() {
                    if !c.is_control() {
                        if state.selected_all {
                            state.input = c.to_string();
                            state.selected_all = false;
                        } else {
                            state.input.push(c);
                        }
                    }
                }

                match super::clipboard::handle(rl, &state.input, |c| !c.is_control()) {
                    ClipboardAction::Replace(s) => {
                        state.input = s;
                        state.selected_all = false;
                    }
                    ClipboardAction::SelectAll => {
                        state.selected_all = true;
                    }
                    ClipboardAction::None => {}
                }

                if rl.is_key_pressed(KeyboardKey::KEY_BACKSPACE) || rl.is_key_pressed_repeat(KeyboardKey::KEY_BACKSPACE)
                {
                    if state.selected_all {
                        state.input.clear();
                        state.selected_all = false;
                    } else {
                        state.input.pop();
                    }
                }

                if rl.is_key_pressed(KeyboardKey::KEY_ENTER) {
                    state.advance();
                }
            }
        }

        if rl.window_should_close() {
            std::process::exit(0);
        }

        let mut d = rl.begin_drawing(thread);
        d.clear_background(BG);

        match &state.step {
            Step::Confirm => draw_confirm(&mut d, fonts, &state),
            _ => draw_input(&mut d, fonts, &state),
        }
    }
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

fn draw_input(d: &mut RaylibDrawHandle, fonts: &Fonts, state: &SetupState) {
    let (title, sub) = step_labels(&state.step);

    txt(d, fonts, title, PAD, PAD, FONT_SIZE + 2.0, FG);
    txt(d, fonts, sub, PAD, PAD + 32, FONT_SIZE - 2.0, DIM);

    let by = PAD + 70;
    let rect = Rectangle {
        x: PAD as f32,
        y: by as f32,
        width: INPUT_W as f32,
        height: INPUT_H as f32,
    };
    d.draw_rectangle_rec(rect, BOX_ACT);
    d.draw_rectangle_lines_ex(rect, 1.5, ACCENT);

    let ty = by + (INPUT_H - FONT_SIZE as i32) / 2;

    // Draw selection highlight behind the text when select-all is active.
    if state.selected_all && !state.input.is_empty() {
        let sel_w = measure(fonts, &state.input, FONT_SIZE);
        d.draw_rectangle(PAD + 8, ty, sel_w, FONT_SIZE as i32, SEL_BG);
    }

    let (text, color) = if state.input.is_empty() {
        ("type and press Enter", DIM)
    } else {
        (state.input.as_str(), FG)
    };
    txt(d, fonts, text, PAD + 8, ty, FONT_SIZE, color);

    // Only show the blinking cursor when not in select-all mode.
    if !state.selected_all && (d.get_time() * 2.0) as i32 % 2 == 0 {
        let cx = PAD + 8 + measure(fonts, &state.input, FONT_SIZE) + 1;
        txt(d, fonts, "|", cx, ty, FONT_SIZE, ACCENT);
    }

    if let Some(err) = &state.error {
        txt(d, fonts, err, PAD, by + INPUT_H + 12, FONT_SIZE - 2.0, RED);
    }

    txt(
        d,
        fonts,
        "Press Enter to continue",
        PAD,
        INPUT_WIN_H - PAD - FONT_SIZE as i32,
        FONT_SIZE - 4.0,
        DIM,
    );
}

fn draw_confirm(d: &mut RaylibDrawHandle, fonts: &Fonts, state: &SetupState) {
    txt(d, fonts, "Confirm config", PAD, PAD, FONT_SIZE + 2.0, FG);

    let line_h = FONT_SIZE as i32 + 6;
    let mut y = PAD + 36;

    for (label, value, color) in [
        ("Group:  ", state.group_name.as_str(), DIM),
        ("You:    ", state.my_name.as_str(), DIM),
    ] {
        txt(d, fonts, &format!("{label}{value}"), PAD, y, FONT_SIZE - 2.0, color);
        y += line_h;
    }

    txt(d, fonts, "Members:", PAD, y, FONT_SIZE - 2.0, DIM);
    y += line_h;

    for (i, name) in state.members.iter().enumerate() {
        let is_me = name.trim().eq_ignore_ascii_case(&state.my_name);
        let label = if is_me {
            format!("  {}. {} (you)", i + 1, name)
        } else {
            format!("  {}. {}", i + 1, name)
        };
        txt(d, fonts, &label, PAD, y, FONT_SIZE - 2.0, FG);
        y += line_h;
    }

    txt(
        d,
        fonts,
        "[Y] Save and continue   [N] Start over",
        PAD,
        y + 6,
        FONT_SIZE - 2.0,
        GREEN,
    );
}

fn step_labels(step: &Step) -> (&'static str, &'static str) {
    match step {
        Step::GroupName => ("Setup — Group name", "Enter your group's name"),
        Step::MyName => ("Setup — Your name", "Enter your full name as registered at LNU"),
        Step::MemberCount => ("Setup — Team size", "How many members in total? (1-7)"),
        Step::MemberName(_) => ("Setup — Member name", "Enter this member's full name"),
        Step::Confirm => ("", ""),
    }
}

fn write_config(path: &Path, config: &Config) -> std::io::Result<()> {
    let students = config
        .members
        .students
        .iter()
        .map(|s| format!("    \"{s}\","))
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!(
        "[general]\ngroup_name = \"{}\"\nmy_name    = \"{}\"\n\n[members]\nstudents = [\n{students}\n]\n",
        config.general.group_name, config.general.my_name,
    );

    std::fs::write(path, content)
}
