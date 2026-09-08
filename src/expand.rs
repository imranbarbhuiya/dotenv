use crate::EnvMap;
use std::env;
use std::fmt;
use std::io;
use std::process::Command;

#[derive(Debug, Clone, Copy)]
pub struct ExpandOptions {
    /// Evaluate `$(command)` expressions. This matches dotenv-expand 1000.0.0.
    /// Disable it for files that are not fully trusted.
    pub command_substitution: bool,
    /// Let non-empty values inherited from the process take priority.
    pub process_env_precedence: bool,
}

impl Default for ExpandOptions {
    fn default() -> Self {
        Self {
            command_substitution: true,
            process_env_precedence: true,
        }
    }
}

#[derive(Debug)]
pub enum ExpandError {
    UnclosedExpression,
    RequiredVariable {
        name: String,
        message: String,
    },
    CommandIo(io::Error),
    CommandFailed {
        command: String,
        status: Option<i32>,
    },
    Decryption(String),
}

impl fmt::Display for ExpandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnclosedExpression => f.write_str("unclosed expansion expression"),
            Self::RequiredVariable { name, message } => {
                write!(f, "required variable {name} is missing or empty: {message}")
            }
            Self::CommandIo(error) => write!(f, "command substitution failed: {error}"),
            Self::CommandFailed { command, status } => {
                write!(f, "command substitution `{command}` exited with {status:?}")
            }
            Self::Decryption(message) => write!(f, "could not decrypt dotenv value: {message}"),
        }
    }
}

impl std::error::Error for ExpandError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::CommandIo(error) => Some(error),
            _ => None,
        }
    }
}

/// Expand a parsed map using the current process environment.
pub fn expand(parsed: &EnvMap) -> Result<EnvMap, ExpandError> {
    let process_env = env::vars().collect::<EnvMap>();
    expand_with(parsed, &process_env, ExpandOptions::default())
}

/// Expand variables progressively in insertion order.
///
/// Supports `$NAME`, `${NAME}`, `${NAME:-default}`, `${NAME-default}`,
/// `${NAME:+alternate}`, `${NAME+alternate}`, and the `:?`/`?` required forms.
pub fn expand_with(
    parsed: &EnvMap,
    process_env: &EnvMap,
    options: ExpandOptions,
) -> Result<EnvMap, ExpandError> {
    let mut running = EnvMap::with_capacity(parsed.len());
    for (key, literal) in parsed {
        let inherited = process_env.get(key).filter(|value| !value.is_empty());
        let mut value =
            if options.process_env_precedence && inherited.is_some_and(|value| value != literal) {
                inherited.cloned().unwrap_or_default()
            } else {
                let decrypted;
                let literal = if literal.starts_with("encrypted:") {
                    let private_key = process_env
                        .get("DOTENV_PRIVATE_KEY")
                        .or_else(|| running.get("DOTENV_PRIVATE_KEY"));
                    if let Some(private_key) = private_key {
                        decrypted = dotenvx_primitives::decrypt(private_key, literal)
                            .map_err(|error| ExpandError::Decryption(error.to_string()))?;
                        &decrypted
                    } else {
                        literal
                    }
                } else {
                    literal
                };
                let mut stack = vec![key.clone()];
                expand_value(literal, key, parsed, process_env, &running, &mut stack)?
            };
        if options.command_substitution {
            value = evaluate_commands(&value)?;
        }
        running.insert(key.clone(), value);
    }
    Ok(running)
}

fn expand_value(
    input: &str,
    current: &str,
    literals: &EnvMap,
    process_env: &EnvMap,
    running: &EnvMap,
    stack: &mut Vec<String>,
) -> Result<String, ExpandError> {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut copy_start = 0;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'\\' && bytes.get(i + 1) == Some(&b'$') {
            output.push_str(&input[copy_start..i]);
            output.push('$');
            i += 2;
            copy_start = i;
            continue;
        }
        if bytes[i] != b'$' {
            i += 1;
            continue;
        }

        if bytes.get(i + 1) == Some(&b'{') {
            let end = matching_brace(input, i + 1)?;
            output.push_str(&input[copy_start..i]);
            let expression = &input[i + 2..end];
            output.push_str(&expand_braced(
                expression,
                current,
                literals,
                process_env,
                running,
                stack,
            )?);
            i = end + 1;
            copy_start = i;
            continue;
        }

        let name_start = i + 1;
        if bytes
            .get(name_start)
            .is_none_or(|byte| !is_variable_start(*byte))
        {
            i += 1;
            continue;
        }
        let mut end = name_start;
        while end < bytes.len() && is_variable_byte(bytes[end]) {
            end += 1;
        }
        if end == name_start {
            i += 1;
            continue;
        }
        output.push_str(&input[copy_start..i]);
        let name = &input[name_start..end];
        output.push_str(
            &resolve(name, current, literals, process_env, running, stack)?.unwrap_or_default(),
        );
        i = end;
        copy_start = i;
    }
    output.push_str(&input[copy_start..]);
    Ok(output)
}

fn expand_braced(
    expression: &str,
    current: &str,
    literals: &EnvMap,
    process_env: &EnvMap,
    running: &EnvMap,
    stack: &mut Vec<String>,
) -> Result<String, ExpandError> {
    let (name, operator, operand) = split_expression(expression);
    let resolved = resolve(name, current, literals, process_env, running, stack)?;
    let is_set = resolved.is_some();
    let is_nonempty = resolved.as_ref().is_some_and(|value| !value.is_empty());
    let expand_operand = || {
        let mut nested_stack = stack.clone();
        expand_value(
            operand,
            current,
            literals,
            process_env,
            running,
            &mut nested_stack,
        )
    };

    match operator {
        None => Ok(resolved.unwrap_or_default()),
        Some(":-") if !is_nonempty => expand_operand(),
        Some("-") if !is_set => expand_operand(),
        Some(":+") if is_nonempty => expand_operand(),
        Some("+") if is_set => expand_operand(),
        Some(":+") | Some("+") => Ok(String::new()),
        Some(":?") if !is_nonempty => Err(ExpandError::RequiredVariable {
            name: name.to_owned(),
            message: operand.to_owned(),
        }),
        Some("?") if !is_set => Err(ExpandError::RequiredVariable {
            name: name.to_owned(),
            message: operand.to_owned(),
        }),
        Some(_) => Ok(resolved.unwrap_or_default()),
    }
}

fn resolve(
    name: &str,
    current: &str,
    literals: &EnvMap,
    process_env: &EnvMap,
    running: &EnvMap,
    stack: &mut Vec<String>,
) -> Result<Option<String>, ExpandError> {
    if name == current || stack.iter().any(|item| item == name) {
        return Ok(None);
    }
    if let Some(process_value) = process_env.get(name) {
        if literals.get(name) == Some(process_value) {
            if let Some(running_value) = running.get(name) {
                return Ok(Some(running_value.clone()));
            }
        }
        stack.push(name.to_owned());
        let value = expand_value(
            process_value,
            current,
            literals,
            process_env,
            running,
            stack,
        )?;
        stack.pop();
        return Ok(Some(value));
    }
    Ok(running.get(name).cloned())
}

fn split_expression(expression: &str) -> (&str, Option<&str>, &str) {
    let bytes = expression.as_bytes();
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => depth = depth.saturating_sub(1),
            b':' if depth == 0 && matches!(bytes.get(i + 1), Some(b'-' | b'+' | b'?')) => {
                return (
                    &expression[..i],
                    Some(&expression[i..i + 2]),
                    &expression[i + 2..],
                );
            }
            b'-' | b'+' | b'?' if depth == 0 => {
                return (
                    &expression[..i],
                    Some(&expression[i..i + 1]),
                    &expression[i + 1..],
                );
            }
            _ => {}
        }
        i += 1;
    }
    (expression, None, "")
}

fn evaluate_commands(input: &str) -> Result<String, ExpandError> {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut copy_start = 0;
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] != b'$' || bytes[i + 1] != b'(' {
            i += 1;
            continue;
        }
        let mut depth = 1usize;
        let mut end = i + 2;
        while end < bytes.len() && depth != 0 {
            match bytes[end] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                _ => {}
            }
            end += 1;
        }
        if depth != 0 {
            return Err(ExpandError::UnclosedExpression);
        }
        let command = &input[i + 2..end - 1];
        let command_output = run_shell(command)?;
        output.push_str(&input[copy_start..i]);
        output.push_str(command_output.trim_end_matches(['\r', '\n']));
        i = end;
        copy_start = i;
    }
    output.push_str(&input[copy_start..]);
    Ok(output)
}

fn run_shell(command: &str) -> Result<String, ExpandError> {
    #[cfg(windows)]
    let output = Command::new("cmd").args(["/C", command]).output();
    #[cfg(not(windows))]
    let output = Command::new("sh").args(["-c", command]).output();

    let output = output.map_err(ExpandError::CommandIo)?;
    if !output.status.success() {
        return Err(ExpandError::CommandFailed {
            command: command.to_owned(),
            status: output.status.code(),
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn matching_brace(input: &str, open: usize) -> Result<usize, ExpandError> {
    let bytes = input.as_bytes();
    let mut depth = 1usize;
    let mut i = open + 1;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Ok(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Err(ExpandError::UnclosedExpression)
}

#[inline]
fn is_variable_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.')
}

#[inline]
fn is_variable_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || byte == b'_'
}
