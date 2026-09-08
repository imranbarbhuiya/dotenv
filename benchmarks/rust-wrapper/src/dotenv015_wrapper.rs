use std::env;
use std::process::{Command, ExitCode};

#[cfg(unix)]
use std::os::unix::process::CommandExt;

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() < 2 {
        eprintln!("usage: dotenv015-wrapper <file> <command> [args...]");
        return ExitCode::from(2);
    }
    #[allow(deprecated)]
    let values = match dotenv_upstream::from_path_iter(&args[0]) {
        Ok(values) => match values.collect::<Result<Vec<_>, _>>() {
            Ok(values) => values,
            Err(error) => {
                eprintln!("{error}");
                return ExitCode::FAILURE;
            }
        },
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    let mut command = Command::new(&args[1]);
    command.args(&args[2..]).envs(values);
    #[cfg(unix)]
    {
        let error = command.exec();
        eprintln!("{error}");
        ExitCode::FAILURE
    }
    #[cfg(not(unix))]
    match command.status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1).clamp(0, 255) as u8),
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
