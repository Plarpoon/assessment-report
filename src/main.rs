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

    let config = toml::parser::load(&binary_dir.join("config.toml")).unwrap_or_else(|e| {
        eprintln!("{e}");
        process::exit(1)
    });

    let assignments = veuros::run(&config);

    let my_name = config.general.my_name.trim();
    let week = Local::now().iso_week().week();
    let group = config.general.group_name.trim().replace(' ', "_");
    let name_slug = my_name.replace(' ', "_");
    let filename = format!("1DV508WEEK{week}{group}By{name_slug}.txt");

    let content = build_content(my_name, &config.members.students, &assignments);
    fs::write(binary_dir.join(&filename), &content)
        .unwrap_or_else(|e| eprintln!("Error writing file: {e}"));

    println!("Written: {filename}");
}

fn build_content(my_name: &str, students: &[String], assignments: &[veuros::Assignment]) -> String {
    students
        .iter()
        .map(|student| {
            let s = student.trim();
            if s.eq_ignore_ascii_case(my_name) {
                "ME, -".to_string()
            } else {
                let amount = assignments
                    .iter()
                    .find(|a| a.name.eq_ignore_ascii_case(s))
                    .map(|a| a.amount.to_string())
                    .unwrap_or_default();
                format!("{s}, {amount}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}
