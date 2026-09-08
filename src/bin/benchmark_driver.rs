use dotenv::{DotenvEditor, parse};
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
        "extra" => {
            for _ in 0..iterations {
                let mut editor = match DotenvEditor::open(black_box(&path)) {
                    Ok(editor) => editor,
                    Err(error) => {
                        eprintln!("{error}");
                        return ExitCode::FAILURE;
                    }
                };
                if editor.upsert("BENCHMARK_TARGET", "updated value").is_err() {
                    return ExitCode::FAILURE;
                }
                checksum = checksum.wrapping_add(black_box(editor.dump()).len());
                checksum = checksum.wrapping_add(black_box(editor.render()).len());
            }
        }
        _ => return usage(),
    }
    println!("{checksum}");
    ExitCode::SUCCESS
}

fn usage() -> ExitCode {
    eprintln!("usage: dotenv-bench-driver <parse|extra> <file> [iterations]");
    ExitCode::FAILURE
}
