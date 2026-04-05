use serde::Deserialize;
use std::{collections::HashSet, path::Path};

use super::defaults;

#[derive(Deserialize)]
pub struct General {
    pub group_name: String,
    pub my_name: String,
}

#[derive(Deserialize)]
pub struct Members {
    pub students: Vec<String>,
}

#[derive(Deserialize)]
pub struct Config {
    pub general: General,
    pub members: Members,
}

#[derive(Debug)]
pub enum ConfigError {
    Missing,
    Io(std::io::Error),
    Parse(toml::de::Error),
    Semantic(String),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Missing => write!(
                f,
                "config not found — a default has been written, fill it in and run again"
            ),
            ConfigError::Io(e) => write!(f, "could not read config: {e}"),
            ConfigError::Parse(e) => write!(f, "malformed config: {e}"),
            ConfigError::Semantic(s) => write!(f, "{s}"),
        }
    }
}

pub fn load(path: &Path) -> Result<Config, ConfigError> {
    // First run: generate a template the user can fill in, then exit.
    if defaults::is_missing(path) {
        defaults::write(path).map_err(ConfigError::Io)?;
        return Err(ConfigError::Missing);
    }

    let raw = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
    let config = toml::from_str(&raw).map_err(ConfigError::Parse)?;

    validate(&config)?;
    Ok(config)
}

fn validate(config: &Config) -> Result<(), ConfigError> {
    let students = &config.members.students;
    let my_name = config.general.my_name.trim();
    let group_name = config.general.group_name.trim();

    if students.is_empty() || students.len() > 7 {
        return Err(ConfigError::Semantic(format!(
            "[members] students must have 1-7 entries (found {}).",
            students.len()
        )));
    }

    let mut seen = HashSet::new();
    for name in students {
        if !seen.insert(name.trim().to_lowercase()) {
            return Err(ConfigError::Semantic(format!("duplicate student name: \"{name}\"")));
        }
    }

    if !students.iter().any(|s| s.trim().eq_ignore_ascii_case(my_name)) {
        return Err(ConfigError::Semantic(format!(
            "my_name \"{my_name}\" is not listed under [members] students"
        )));
    }

    if group_name == "group name" || my_name == "name surname 1" {
        return Err(ConfigError::Semantic(
            "config still contains placeholder values — update group_name and my_name".into(),
        ));
    }

    Ok(())
}
