use crate::{EnvMap, parse};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, Default)]
pub struct LoadOptions {
    /// Replace variables already present in the destination.
    pub override_existing: bool,
}

/// Find `.env` in `start` or its nearest ancestor.
#[must_use]
pub fn find_dotenv(start: impl AsRef<Path>) -> Option<PathBuf> {
    let mut current = start.as_ref();
    if current.is_file() {
        current = current.parent()?;
    }
    loop {
        let candidate = current.join(".env");
        if candidate.is_file() {
            return Some(candidate);
        }
        current = current.parent()?;
    }
}

/// Parse a specific UTF-8 dotenv file without modifying the process environment.
pub fn load_from_path(path: impl AsRef<Path>) -> io::Result<EnvMap> {
    fs::read_to_string(path).map(|text| parse(&text))
}

/// Find and parse the nearest `.env` from the current directory upward.
pub fn load() -> io::Result<EnvMap> {
    let path = find_dotenv(env::current_dir()?)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "could not find .env"))?;
    load_from_path(path)
}

/// Merge parsed values into any dotenv map.
pub fn populate(destination: &mut EnvMap, parsed: EnvMap, options: LoadOptions) {
    if options.override_existing {
        destination.extend(parsed);
    } else {
        for (key, value) in parsed {
            destination.entry(key).or_insert(value);
        }
    }
}
