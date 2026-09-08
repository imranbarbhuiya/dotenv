use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
    let count = env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(0);
    for index in 0..count {
        let key = format!("BENCH_KEY_{index}");
        let expected = format!("value-{index}");
        if env::var(&key).as_deref() != Ok(expected.as_str()) {
            eprintln!("incorrect or missing value for {key}");
            return ExitCode::FAILURE;
        }
    }
    println!("verified {count} variables");
    ExitCode::SUCCESS
}
