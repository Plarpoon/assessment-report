use std::{fs, path::Path};

const DEFAULT_CONFIG: &str = r#"[general]
group_name = "group name"
my_name    = "name surname 1"

[members]
students = [
    "name surname 1",
    "name surname 2",
    "name surname 3",
    "name surname 4",
]
"#;

pub fn write(path: &Path) {
    fs::write(path, DEFAULT_CONFIG)
        .unwrap_or_else(|e| eprintln!("Warning: could not write default config: {e}"));
}

pub fn is_missing(path: &Path) -> bool {
    !path.exists()
}
