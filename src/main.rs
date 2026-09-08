use dotenv::{EnvMap, ExpandOptions, expand_with, load, load_from_path};
use std::env;
use std::io::{self, BufWriter, Write};
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

    let parsed = if paths.is_empty() {
        load()?
    } else {
        let mut paths = paths.into_iter();
        // Keep the first file's allocation instead of rehashing every entry.
        let mut parsed = load_from_path(paths.next().expect("nonempty paths"))?;
        for path in paths {
            for (key, value) in load_from_path(path)? {
                parsed.insert(key, value);
            }
        }
        parsed
    };

    let values = if should_expand {
        let process_env = env::vars().collect::<EnvMap>();
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
        for (key, inherited) in env::vars() {
            if let Some(value) = values.get_mut(&key) {
                *value = inherited;
            }
        }
        values
    };

    if command.is_empty() {
        let stdout = io::stdout();
        let mut output = BufWriter::new(stdout.lock());
        for (key, value) in values {
            writeln!(output, "{key}={value}")?;
        }
        output.flush()?;
        return Ok(0);
    }

    let mut child = Command::new(&command[0]);
    child.args(&command[1..]).envs(values);
    #[cfg(unix)]
    {
        // Replace the loader rather than retaining an idle parent process.
        // This also preserves child exit signals and normal signal delivery.
        use std::os::unix::process::CommandExt;
        Err(child.exec().into())
    }
    #[cfg(not(unix))]
    {
        let status = child.status()?;
        Ok(status.code().unwrap_or(1).clamp(0, 255) as u8)
    }
}

fn print_help() {
    println!(
        "dotenv {version}\n\nUSAGE:\n    dotenv [OPTIONS] [--] [COMMAND [ARGS...]]\n\nOPTIONS:\n    -f, --file PATH                Load PATH; repeat to layer files\n        --override                 File values replace inherited variables\n        --no-expand                Disable variable expansion\n        --no-command-substitution  Disable $(command) evaluation\n    -h, --help                     Print help\n    -V, --version                  Print version\n\nWith no command, the resulting entries are printed. Dotenv files must be trusted\nwhen command substitution is enabled.",
        version = env!("CARGO_PKG_VERSION")
    );
}
