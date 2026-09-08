use indexmap::IndexMap;
use std::borrow::Cow;

/// Parsed environment entries. As in upstream `dotenv`, the last duplicate wins.
pub type EnvMap = IndexMap<String, String>;

/// Parse UTF-8 dotenv text with the syntax supported by `motdotla/dotenv`.
#[must_use]
pub fn parse(input: &str) -> EnvMap {
    let normalized = normalize_newlines(input);
    parse_normalized(normalized.as_ref())
}

/// Parse UTF-8 bytes. A UTF-8 BOM is accepted; invalid UTF-8 is rejected.
pub fn parse_bytes(input: &[u8]) -> Result<EnvMap, std::str::Utf8Error> {
    std::str::from_utf8(input).map(parse)
}

fn normalize_newlines(input: &str) -> Cow<'_, str> {
    if !input.as_bytes().contains(&b'\r') {
        return Cow::Borrowed(input);
    }
    let mut output = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\r' {
            output.push_str(&input[start..i]);
            output.push('\n');
            i += 1;
            if i < bytes.len() && bytes[i] == b'\n' {
                i += 1;
            }
            start = i;
        } else {
            i += 1;
        }
    }
    output.push_str(&input[start..]);
    Cow::Owned(output)
}

#[inline]
fn is_key_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.')
}

#[inline]
fn is_horizontal_space(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t')
}

fn parse_normalized(input: &str) -> EnvMap {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let mut entries = EnvMap::new();
    let mut i = 0;

    while i < len {
        while i < len
            && (matches!(bytes[i], b' ' | b'\t' | b'\n') || input[i..].starts_with('\u{feff}'))
        {
            i += if input[i..].starts_with('\u{feff}') {
                3
            } else {
                1
            };
        }
        if i >= len {
            break;
        }
        if bytes[i] == b'#' {
            i = skip_line(bytes, i);
            continue;
        }
        if bytes[i..].starts_with(b"export")
            && bytes
                .get(i + 6)
                .is_some_and(|byte| is_horizontal_space(*byte))
        {
            i += 6;
            while i < len && is_horizontal_space(bytes[i]) {
                i += 1;
            }
        }

        let key_start = i;
        while i < len && is_key_byte(bytes[i]) {
            i += 1;
        }
        if i == key_start {
            i = skip_line(bytes, i);
            continue;
        }
        let key_end = i;
        while i < len && is_horizontal_space(bytes[i]) {
            i += 1;
        }
        let valid_separator = bytes.get(i) == Some(&b'=')
            || (bytes.get(i) == Some(&b':')
                && bytes
                    .get(i + 1)
                    .is_some_and(|byte| is_horizontal_space(*byte)));
        if valid_separator {
            i += 1;
        } else {
            i = skip_line(bytes, i);
            continue;
        }
        while i < len && is_horizontal_space(bytes[i]) {
            i += 1;
        }

        let value = if matches!(bytes.get(i), Some(b'\'' | b'"' | b'`')) {
            parse_quoted(input, &mut i)
        } else {
            parse_unquoted(input, &mut i)
        };
        entries.insert(input[key_start..key_end].to_owned(), value);
    }
    entries
}

fn parse_quoted(input: &str, i: &mut usize) -> String {
    let bytes = input.as_bytes();
    let len = bytes.len();
    let quote = bytes[*i];
    let value_start = *i + 1;
    let mut cursor = value_start;
    while cursor < len {
        if bytes[cursor] == b'\\'
            && cursor + 1 < len
            && (bytes[cursor + 1] == quote || bytes[cursor + 1] == b'\\')
        {
            cursor += 2;
            continue;
        }
        if bytes[cursor] == quote {
            let raw = &input[value_start..cursor];
            *i = cursor + 1;
            while *i < len && is_horizontal_space(bytes[*i]) {
                *i += 1;
            }
            if bytes.get(*i) == Some(&b'#') {
                *i = skip_line(bytes, *i);
            }
            return if quote == b'"' {
                expand_newlines(raw)
            } else {
                raw.to_owned()
            };
        }
        cursor += 1;
    }
    *i = value_start - 1;
    parse_unquoted(input, i)
}

fn parse_unquoted(input: &str, i: &mut usize) -> String {
    let bytes = input.as_bytes();
    let start = *i;
    while *i < bytes.len() && !matches!(bytes[*i], b'#' | b'\n') {
        *i += 1;
    }
    let mut end = *i;
    while end > start && is_horizontal_space(bytes[end - 1]) {
        end -= 1;
    }
    if bytes.get(*i) == Some(&b'#') {
        *i = skip_line(bytes, *i);
    }
    input[start..end].to_owned()
}

fn expand_newlines(raw: &str) -> String {
    if !raw
        .as_bytes()
        .windows(2)
        .any(|pair| matches!(pair, b"\\n" | b"\\r"))
    {
        return raw.to_owned();
    }
    let bytes = raw.as_bytes();
    let mut output = String::with_capacity(raw.len());
    let mut start = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == b'\\' && matches!(bytes[i + 1], b'n' | b'r') {
            output.push_str(&raw[start..i]);
            output.push(if bytes[i + 1] == b'n' { '\n' } else { '\r' });
            i += 2;
            start = i;
        } else {
            i += 1;
        }
    }
    output.push_str(&raw[start..]);
    output
}

#[inline]
fn skip_line(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}
