use dotenv::{EnvMap, ExpandOptions, expand_with, load, load_from_path};
use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(code) => ExitCode::from(code),
        Err(error) => {
            eprintln!("dotenv: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<u8, Box<dyn std::error::Error>> {
    let mut paths = Vec::<PathBuf>::new();
    let mut override_existing = false;
    let mut should_expand = true;
    let mut command_substitution = true;
    let mut command = Vec::new();
    let mut args = env::args().skip(1).peekable();

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "-f" | "--file" => {
                paths.push(args.next().ok_or("--file requires a path")?.into());
            }
            "--override" => override_existing = true,
            "--no-expand" => should_expand = false,
            "--no-command-substitution" => command_substitution = false,
            "-h" | "--help" => {
                print_help();
                return Ok(0);
            }
            "-V" | "--version" => {
                println!("dotenv {}", env!("CARGO_PKG_VERSION"));
                return Ok(0);
            }
            "--" => {
                command.extend(args);
                break;
            }
            _ if argument.starts_with('-') => {
                return Err(format!("unknown option: {argument}").into());
            }
            _ => {
                command.push(argument);
                command.extend(args);
                break;
            }
        }
    }

    let mut parsed = EnvMap::new();
    if paths.is_empty() {
        parsed = load()?;
    } else {
        for path in paths {
            for (key, value) in load_from_path(path)? {
                parsed.insert(key, value);
            }
        }
    }

    let process_env = env::vars().collect::<EnvMap>();
    let values = if should_expand {
        expand_with(
            &parsed,
            &process_env,
            ExpandOptions {
                command_substitution,
                process_env_precedence: !override_existing,
            },
        )?
    } else if override_existing {
        parsed
    } else {
        let mut values = parsed;
        for (key, value) in &process_env {
            if values.contains_key(key) {
                values.insert(key.clone(), value.clone());
            }
        }
        values
    };

    if command.is_empty() {
        for (key, value) in values {
            println!("{key}={value}");
        }
        return Ok(0);
    }

    let status = Command::new(&command[0])
        .args(&command[1..])
        .envs(values)
        .status()?;
    Ok(status.code().unwrap_or(1).clamp(0, 255) as u8)
}

fn print_help() {
    println!(
        "dotenv {version}\n\nUSAGE:\n    dotenv [OPTIONS] [--] [COMMAND [ARGS...]]\n\nOPTIONS:\n    -f, --file PATH                Load PATH; repeat to layer files\n        --override                 File values replace inherited variables\n        --no-expand                Disable variable expansion\n        --no-command-substitution  Disable $(command) evaluation\n    -h, --help                     Print help\n    -V, --version                  Print version\n\nWith no command, the resulting entries are printed. Dotenv files must be trusted\nwhen command substitution is enabled.",
        version = env!("CARGO_PKG_VERSION")
    );
}
