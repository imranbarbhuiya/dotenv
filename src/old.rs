use std::collections::HashMap;
mod parser;

#[derive(Debug, PartialEq)]
pub enum Token {
    Key(String),
    Equals,
    Value(String),
    Comment(String),
    Newline,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut iter = input.lines();
    while let Some(line) = iter.next() {
        let line = line.trim();
        if line.starts_with('#') {
            tokens.push(Token::Comment(line.to_string()));
        } else if line.contains('=') {
            let mut parts = line.splitn(2, '=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                tokens.push(Token::Key(key.trim().to_string()));
                tokens.push(Token::Equals);

                // Check for multiline values
                let mut value = value.trim().to_string();
                if value.starts_with('"') && !value.ends_with('"') {
                    value.push('\n');
                    for next_line in iter.by_ref() {
                        value.push_str(next_line);
                        value.push('\n');
                        if next_line.trim().ends_with('"') {
                            break;
                        }
                    }
                }

                tokens.push(Token::Value(value));
            }
        }
        tokens.push(Token::Newline);
    }
    tokens
}

pub fn parse_tokens(tokens: Vec<Token>) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let mut key = String::new();
    let mut value = String::new();
    let mut in_value = false;

    for token in tokens {
        match token {
            Token::Key(k) => {
                if in_value {
                    map.insert(key.clone(), substitute_variables(&value, &map));
                    value.clear();
                }
                key = k;
                in_value = false;
            }
            Token::Equals => {
                in_value = true;
            }
            Token::Value(v) => {
                if in_value {
                    value = v;
                }
            }
            Token::Newline => {
                if in_value {
                    map.insert(key.clone(), substitute_variables(&value, &map));
                    key.clear();
                    value.clear();
                    in_value = false;
                }
            }
            Token::Comment(_) => {} // Ignore comments
        }
    }

    if !key.is_empty() && !value.is_empty() {
        map.insert(key, substitute_variables(&value, &map));
    }

    map
}

fn substitute_variables(value: &str, map: &HashMap<String, String>) -> String {
    let mut result = value.to_string();
    let mut replacements: Vec<(String, String)> = Vec::new();

    // Collect tokens and their replacements
    let tokens = result.split_whitespace().collect::<Vec<&str>>();
    for token in tokens {
        if token.starts_with('$') {
            let key = token.strip_prefix('$').unwrap();
            if let Some(replacement) = map.get(key) {
                replacements.push((token.to_string(), replacement.clone()));
            }
        }
    }

    // Apply replacements
    for (token, replacement) in replacements {
        result = result.replace(&token, &replacement);
    }

    // Trim double quotes if they enclose the whole string
    trim_quotes(&result)
}

fn trim_quotes(value: &str) -> String {
    let mut trimmed = value.to_string();
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() > 1 {
        trimmed = trimmed[1..trimmed.len() - 1].to_string();
    }
    trimmed
}
