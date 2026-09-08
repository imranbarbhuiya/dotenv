use dotenv::{EnvMap, ExpandOptions, expand_with, parse};
use std::env;
use std::fs;
use std::hint::black_box;
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = env::args().skip(1);
    let Some(mode) = args.next() else {
        return usage();
    };
    let Some(path) = args.next() else {
        return usage();
    };
    let iterations = args
        .next()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(1_000);
    let mut checksum = 0usize;

    match mode.as_str() {
        "parse" => {
            let input = match fs::read_to_string(path) {
                Ok(input) => input,
                Err(error) => {
                    eprintln!("{error}");
                    return ExitCode::FAILURE;
                }
            };
            for _ in 0..iterations {
                checksum = checksum.wrapping_add(black_box(parse(black_box(&input))).len());
            }
        }
        "expand" => {
            let input = match fs::read_to_string(path) {
                Ok(input) => input,
                Err(error) => {
                    eprintln!("{error}");
                    return ExitCode::FAILURE;
                }
            };
            let process_env = EnvMap::new();
            for _ in 0..iterations {
                let parsed = parse(black_box(&input));
                let expanded = match expand_with(
                    &parsed,
                    &process_env,
                    ExpandOptions {
                        command_substitution: false,
                        process_env_precedence: false,
                    },
                ) {
                    Ok(expanded) => expanded,
                    Err(error) => {
                        eprintln!("{error}");
                        return ExitCode::FAILURE;
                    }
                };
                checksum = checksum.wrapping_add(black_box(expanded).len());
            }
        }
        _ => return usage(),
    }
    println!("{checksum}");
    ExitCode::SUCCESS
}

fn usage() -> ExitCode {
    eprintln!("usage: dotenv-bench-driver <parse|expand> <file> [iterations]");
    ExitCode::FAILURE
}
