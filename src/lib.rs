//! Fast dotenv parsing, loading, and variable expansion.

mod expand;
mod loader;
mod parser;

pub use expand::{ExpandError, ExpandOptions, expand, expand_with};
pub use loader::{LoadOptions, find_dotenv, load, load_from_path, populate};
pub use parser::{EnvMap, parse, parse_bytes};
