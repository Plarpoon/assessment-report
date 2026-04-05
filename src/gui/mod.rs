mod app;
mod setup;

use std::path::PathBuf;

use raylib::prelude::*;

use crate::toml::parser::Config;
use crate::veuros::Assignment;

pub const WIN_W: i32 = 520;

static FONT_BYTES: &[u8] = include_bytes!("../assets/NotoSans-Regular.ttf");
static ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

pub fn run(config: Option<Config>, config_path: PathBuf) -> (Config, Vec<Assignment>) {
    let (mut rl, thread) = raylib::init()
        .size(WIN_W, 400)
        .title("Peer Assessment Report Generator")
        .build();

    rl.set_target_fps(60);

    let codepoints: String = (0x0020u32..=0x024F) // Basic Latin + Latin Extended A & B
        .chain(0x0370..=0x03FF) // Greek & Coptic
        .chain(0x0400..=0x04FF) // Cyrillic
        .chain(0x2000..=0x206F) // General Punctuation (en/em dash, etc.)
        .filter_map(std::char::from_u32)
        .collect();

    let font = rl
        .load_font_from_memory(&thread, ".ttf", FONT_BYTES, 32, Some(&codepoints))
        .expect("failed to load embedded font");

    if let Ok(icon) = Image::load_image_from_mem(".png", ICON_BYTES) {
        rl.set_window_icon(icon);
    }

    font.texture()
        .set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);

    // If no valid config exists, open the setup wizard before the main screen.
    let mut config = match config {
        Some(c) => c,
        None => setup::run(&mut rl, &thread, &font, &config_path, None),
    };

    loop {
        let outcome = app::run(&mut rl, &thread, &font, &config);

        match outcome {
            app::Outcome::Done(a) => return (config, a),
            app::Outcome::EditConfig => {
                config = setup::run(&mut rl, &thread, &font, &config_path, Some(&config));
            }
        }
    }
}
