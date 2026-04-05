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

pub fn write(path: &Path) -> std::io::Result<()> {
    fs::write(path, DEFAULT_CONFIG)
}

pub fn is_missing(path: &Path) -> bool {
    !path.exists()
}
