mod gui;
mod toml;
mod veuros;

use chrono::{Datelike, Local};
use std::{env, fs, process};

fn main() {
    let binary_dir = env::current_exe()
        .expect("could not resolve binary path")
        .parent()
        .expect("binary has no parent directory")
        .to_path_buf();

    let config_path = binary_dir.join("config.toml");
    let use_console = env::args().any(|a| a == "--console");

    let (config, assignments) = if use_console {
        let config = toml::parser::load(&config_path).unwrap_or_else(|e| {
            eprintln!("{e}");
            process::exit(1)
        });
        let assignments = veuros::run(&config);
        (config, assignments)
    } else {
        gui::run(toml::parser::load(&config_path).ok(), config_path)
    };

    let my_name = config.general.my_name.trim();
    let week = Local::now().iso_week().week();
    let group = config.general.group_name.trim().replace(' ', "_");
    let name_slug = my_name.replace(' ', "");
    let filename = format!("1DV508Week{week}Group{group}By{name_slug}.txt");

    let content = build_content(my_name, &config.members.students, &assignments);

    if let Err(e) = fs::write(binary_dir.join(&filename), &content) {
        eprintln!("Error: could not write '{filename}': {e}");
        process::exit(1);
    }

    println!("Written: {filename}");
}

fn build_content(my_name: &str, students: &[String], assignments: &[veuros::Assignment]) -> String {
    students
        .iter()
        .map(|student| {
            let s = student.trim();
            if s.eq_ignore_ascii_case(my_name) {
                return "ME, -".to_string();
            }
            let amount = assignments
                .iter()
                .find(|a| a.name.eq_ignore_ascii_case(s))
                .map(|a| a.amount.to_string())
                .unwrap_or_default();
            format!("{s}, {amount}")
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
