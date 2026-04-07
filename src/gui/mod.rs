mod app;
pub mod clipboard;
mod setup;

use std::path::PathBuf;

use raylib::prelude::*;

use crate::toml::parser::Config;
use crate::veuros::Assignment;

pub const WIN_W: i32 = 520;

static FONT_BYTES: &[u8] = include_bytes!("../assets/NotoSans-Regular.ttf");
static FONT_CJK_BYTES: &[u8] = include_bytes!("../assets/NotoSansJP-Regular.ttf");
static ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

/// Holds the two font faces used throughout the UI.
/// `pick` returns whichever face covers the given string's script.
pub struct Fonts {
    pub latin: Font,
    pub cjk: Font,
}

impl Fonts {
    /// Returns the CJK font if the string contains any CJK codepoint,
    /// otherwise returns the Latin font.
    pub fn pick(&self, text: &str) -> &Font {
        if text.chars().any(is_cjk) {
            &self.cjk
        } else {
            &self.latin
        }
    }
}

/// Returns true for codepoints in the CJK Unified Ideographs blocks,
/// Hiragana, Katakana, and the Korean Hangul syllables block.
fn is_cjk(c: char) -> bool {
    matches!(c,
        '\u{3040}'..='\u{30FF}' |  // Hiragana + Katakana
        '\u{3400}'..='\u{4DBF}' |  // CJK Extension A
        '\u{4E00}'..='\u{9FFF}' |  // CJK Unified Ideographs
        '\u{AC00}'..='\u{D7AF}' |  // Hangul Syllables
        '\u{F900}'..='\u{FAFF}' |  // CJK Compatibility Ideographs
        '\u{20000}'..='\u{2A6DF}'  // CJK Extension B
    )
}

fn load_font(rl: &mut RaylibHandle, thread: &RaylibThread, bytes: &[u8], codepoints: &str) -> Font {
    let font = rl
        .load_font_from_memory(thread, ".ttf", bytes, 32, Some(codepoints))
        .expect("failed to load embedded font");
    font.texture()
        .set_texture_filter(thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    font
}

pub fn run(config: Option<Config>, config_path: PathBuf) -> (Config, Vec<Assignment>) {
    let (mut rl, thread) = raylib::init()
        .size(WIN_W, 400)
        .title("Peer Assessment Report Generator")
        .build();

    rl.set_target_fps(60);

    // Build codepoint lists for each font face.
    let latin_codepoints: String = (0x0020u32..=0x024F) // Basic Latin + Latin Extended A & B
        .chain(0x0370..=0x03FF)                         // Greek & Coptic
        .chain(0x0400..=0x04FF)                         // Cyrillic
        .chain(0x2000..=0x206F)                         // General Punctuation (en/em dash, etc.)
        .filter_map(std::char::from_u32)
        .collect();

    let cjk_codepoints: String = (0x3040u32..=0x30FF) // Hiragana + Katakana
        .chain(0x3400..=0x4DBF)                        // CJK Extension A
        .chain(0x4E00..=0x9FFF)                        // CJK Unified Ideographs
        .chain(0xAC00..=0xD7AF)                        // Hangul Syllables
        .chain(0xF900..=0xFAFF)                        // CJK Compatibility Ideographs
        .filter_map(std::char::from_u32)
        .collect();

    let fonts = Fonts {
        latin: load_font(&mut rl, &thread, FONT_BYTES, &latin_codepoints),
        cjk: load_font(&mut rl, &thread, FONT_CJK_BYTES, &cjk_codepoints),
    };

    if let Ok(icon) = Image::load_image_from_mem(".png", ICON_BYTES) {
        rl.set_window_icon(icon);
    }

    // If no valid config exists, open the setup wizard before the main screen.
    let mut config = match config {
        Some(c) => c,
        None => setup::run(&mut rl, &thread, &fonts, &config_path, None),
    };

    loop {
        let outcome = app::run(&mut rl, &thread, &fonts, &config);

        match outcome {
            app::Outcome::Done(a) => return (config, a),
            app::Outcome::EditConfig => {
                config = setup::run(&mut rl, &thread, &fonts, &config_path, Some(&config));
            }
        }
    }
}
