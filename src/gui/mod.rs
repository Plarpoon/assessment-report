mod app;
mod setup;

use std::path::PathBuf;

use crate::toml::parser::Config;
use crate::veuros::Assignment;

const WIN_W: i32 = 520;

pub fn run(config: Option<Config>, config_path: PathBuf) -> (Config, Vec<Assignment>) {
    let (mut rl, thread) = raylib::init()
        .size(WIN_W, 400)
        .title("vEuro Assignment")
        .resizable()
        .build();

    rl.set_target_fps(60);

    let mut config = match config {
        Some(c) => c,
        None => setup::run(&mut rl, &thread, &config_path, None),
    };

    loop {
        let assignments = app::run(&mut rl, &thread, &config);

        match assignments {
            app::Outcome::Done(a) => return (config, a),
            app::Outcome::EditConfig => {
                config = setup::run(&mut rl, &thread, &config_path, Some(&config));
            }
        }
    }
}
