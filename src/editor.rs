use crate::EnvMap;
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEnding {
    Lf,
    CrLf,
    Cr,
}

impl LineEnding {
    fn as_str(self) -> &'static str {
        match self {
            Self::Lf => "\n",
            Self::CrLf => "\r\n",
            Self::Cr => "\r",
        }
    }
}

#[derive(Debug)]
pub enum EditorError {
    Io(io::Error),
    MixedLineEndings,
    InvalidKey(&'static str),
    LineBreak,
    UnquotableValue,
}

impl fmt::Display for EditorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => error.fmt(f),
            Self::MixedLineEndings => {
                f.write_str("mixed line endings detected; normalize the file before editing")
            }
            Self::InvalidKey(reason) => write!(f, "invalid key: {reason}"),
            Self::LineBreak => f.write_str("keys and values must not contain line breaks"),
            Self::UnquotableValue => {
                f.write_str("value requires quoting but contains both quote characters")
            }
        }
    }
}

impl std::error::Error for EditorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            _ => None,
        }
    }
}

impl From<io::Error> for EditorError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

#[derive(Debug, Clone, Copy)]
struct Location {
    active: Option<usize>,
    commented: Option<usize>,
}

/// A formatting-preserving dotenv editor compatible with `dotenv-extra`.
#[derive(Debug, Clone)]
pub struct DotenvEditor {
    path: Option<PathBuf>,
    lines: Vec<String>,
    line_ending: LineEnding,
    trailing_newline: bool,
    locations: HashMap<String, Location>,
}

impl DotenvEditor {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, EditorError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let mut editor = Self::from_content(&content)?;
        editor.path = Some(path.to_owned());
        Ok(editor)
    }

    pub fn from_content(content: &str) -> Result<Self, EditorError> {
        let line_ending = detect_line_ending(content)?;
        let separator = line_ending.as_str();
        let trailing_newline = !content.is_empty() && content.ends_with(separator);
        let trimmed = if trailing_newline {
            &content[..content.len() - separator.len()]
        } else {
            content
        };
        let lines = if content.is_empty() {
            Vec::new()
        } else {
            trimmed.split(separator).map(str::to_owned).collect()
        };
        let locations = index_lines(&lines);
        Ok(Self {
            path: None,
            lines,
            line_ending,
            trailing_newline,
            locations,
        })
    }

    #[must_use]
    pub fn lines(&self) -> &[String] {
        &self.lines
    }
    #[must_use]
    pub fn line_ending(&self) -> LineEnding {
        self.line_ending
    }
    #[must_use]
    pub fn has_trailing_newline(&self) -> bool {
        self.trailing_newline
    }

    pub fn upsert(&mut self, key: &str, value: &str) -> Result<&mut Self, EditorError> {
        validate_key_value(key, value)?;
        let rendered = format!("{key}={}", quote_if_needed(value)?);
        let location = self.locations.get(key).copied();
        if let Some(index) = location.and_then(|entry| entry.active.or(entry.commented)) {
            self.lines[index] = rendered;
            self.locations.insert(
                key.to_owned(),
                Location {
                    active: Some(index),
                    commented: None,
                },
            );
        } else {
            let index = self.lines.len();
            self.lines.push(rendered);
            self.locations.insert(
                key.to_owned(),
                Location {
                    active: Some(index),
                    commented: None,
                },
            );
        }
        Ok(self)
    }

    #[must_use]
    pub fn dump(&self) -> EnvMap {
        parse_extra(&self.lines)
    }

    #[must_use]
    pub fn render(&self) -> String {
        if self.lines.is_empty() {
            return String::new();
        }
        let separator = self.line_ending.as_str();
        let capacity = self.lines.iter().map(String::len).sum::<usize>()
            + separator.len() * (self.lines.len() - 1 + usize::from(self.trailing_newline));
        let mut output = String::with_capacity(capacity);
        for (index, line) in self.lines.iter().enumerate() {
            if index != 0 {
                output.push_str(separator);
            }
            output.push_str(line);
        }
        if self.trailing_newline {
            output.push_str(separator);
        }
        output
    }

    pub fn save(&self) -> Result<(), EditorError> {
        let path = self.path.as_ref().ok_or_else(|| {
            EditorError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "editor has no file path",
            ))
        })?;
        fs::write(path, self.render()).map_err(EditorError::Io)
    }

    pub fn save_to(&self, path: impl AsRef<Path>) -> Result<(), EditorError> {
        fs::write(path, self.render()).map_err(EditorError::Io)
    }
}

impl std::str::FromStr for DotenvEditor {
    type Err = EditorError;

    fn from_str(content: &str) -> Result<Self, Self::Err> {
        DotenvEditor::from_content(content)
    }
}

fn detect_line_ending(content: &str) -> Result<LineEnding, EditorError> {
    let bytes = content.as_bytes();
    let (mut lf, mut crlf, mut cr) = (false, false, false);
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\r' if bytes.get(i + 1) == Some(&b'\n') => {
                crlf = true;
                i += 2;
            }
            b'\r' => {
                cr = true;
                i += 1;
            }
            b'\n' => {
                lf = true;
                i += 1;
            }
            _ => i += 1,
        }
    }
    if usize::from(lf) + usize::from(crlf) + usize::from(cr) > 1 {
        return Err(EditorError::MixedLineEndings);
    }
    Ok(if crlf {
        LineEnding::CrLf
    } else if cr {
        LineEnding::Cr
    } else {
        LineEnding::Lf
    })
}

fn index_lines(lines: &[String]) -> HashMap<String, Location> {
    let mut locations = HashMap::new();
    for (index, line) in lines.iter().enumerate() {
        if let Some((key, commented)) = line_key(line) {
            let location = locations.entry(key.to_owned()).or_insert(Location {
                active: None,
                commented: None,
            });
            if commented {
                location.commented = Some(index);
            } else {
                location.active = Some(index);
            }
        }
    }
    locations
}

fn line_key(line: &str) -> Option<(&str, bool)> {
    let mut text = line.trim_start();
    let commented = text.starts_with('#');
    if commented {
        text = text[1..].trim_start();
    }
    let equals = text.find('=')?;
    let key = text[..equals].trim_end();
    (!key.is_empty()).then_some((key, commented))
}

fn validate_key_value(key: &str, value: &str) -> Result<(), EditorError> {
    if key.contains(['\r', '\n']) || value.contains(['\r', '\n']) {
        return Err(EditorError::LineBreak);
    }
    if key.is_empty() || key.trim() != key {
        return Err(EditorError::InvalidKey(
            "must not be empty or padded with whitespace",
        ));
    }
    if key.contains('=') {
        return Err(EditorError::InvalidKey("must not contain '='"));
    }
    if key.starts_with("export ") || key.starts_with("export\t") {
        return Err(EditorError::InvalidKey("must not start with export prefix"));
    }
    Ok(())
}

fn quote_if_needed(value: &str) -> Result<String, EditorError> {
    let needs_quotes = value.starts_with(char::is_whitespace)
        || value.ends_with(char::is_whitespace)
        || value
            .as_bytes()
            .windows(2)
            .any(|pair| pair[0].is_ascii_whitespace() && pair[1] == b'#')
        || value.starts_with(['#', '"', '\'']);
    if !needs_quotes {
        return Ok(value.to_owned());
    }
    let quote = if !value.contains('"') {
        '"'
    } else if !value.contains('\'') {
        '\''
    } else {
        return Err(EditorError::UnquotableValue);
    };
    let trailing_slashes = value
        .as_bytes()
        .iter()
        .rev()
        .take_while(|byte| **byte == b'\\')
        .count();
    let mut output = String::with_capacity(value.len() + trailing_slashes + 2);
    output.push(quote);
    output.push_str(&value[..value.len() - trailing_slashes]);
    output.push_str(&"\\".repeat(trailing_slashes * 2));
    output.push(quote);
    Ok(output)
}

fn parse_extra(lines: &[String]) -> EnvMap {
    let mut result = EnvMap::new();
    for line in lines {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let text = trimmed.strip_prefix("export ").unwrap_or(trimmed);
        let Some(equals) = text.find('=') else {
            continue;
        };
        let key = text[..equals].trim();
        if key.is_empty() {
            continue;
        }
        let mut value = text[equals + 1..].trim();
        let bytes = value.as_bytes();
        if matches!(bytes.first(), Some(b'"' | b'\'')) {
            let quote = bytes[0];
            let mut closing = None;
            let mut slash_run = 0usize;
            for i in 1..bytes.len() {
                if bytes[i] == b'\\' {
                    slash_run += 1;
                    continue;
                }
                if bytes[i] == quote {
                    let remainder = value[i + 1..].trim();
                    if slash_run & 1 == 0 && (remainder.is_empty() || remainder.starts_with('#')) {
                        closing = Some(i);
                        break;
                    }
                }
                slash_run = 0;
            }
            if let Some(end) = closing {
                let quoted = &value[1..end];
                let slashes = quoted
                    .as_bytes()
                    .iter()
                    .rev()
                    .take_while(|byte| **byte == b'\\')
                    .count();
                if slashes == 0 {
                    value = quoted;
                } else {
                    let mut owned = String::with_capacity(quoted.len() - slashes / 2);
                    owned.push_str(&quoted[..quoted.len() - slashes]);
                    owned.push_str(&"\\".repeat(slashes / 2));
                    result.insert(key.to_owned(), owned);
                    continue;
                }
            }
        } else if value.starts_with('#') {
            value = "";
        } else if let Some(comment) = find_inline_comment(value) {
            value = value[..comment].trim_end();
        }
        result.insert(key.to_owned(), value.to_owned());
    }
    result
}

fn find_inline_comment(value: &str) -> Option<usize> {
    value
        .as_bytes()
        .windows(2)
        .position(|pair| pair[0].is_ascii_whitespace() && pair[1] == b'#')
        .map(|index| index + 1)
}
