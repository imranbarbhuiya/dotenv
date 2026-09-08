//! Fast, dependency-free dotenv parsing and formatting-preserving editing.

mod editor;
mod loader;
mod parser;

pub use editor::{DotenvEditor, EditorError, LineEnding};
pub use loader::{LoadOptions, find_dotenv, load, load_from_path, populate};
pub use parser::{EnvMap, parse, parse_bytes};
