use std::process::Command;

#[test]
fn prints_expanded_values_from_an_explicit_file() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("test.env");
    std::fs::write(&path, "CLI_BASE=hello\nCLI_VALUE=${CLI_BASE}-world\n").unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_dotenv"))
        .args(["--file", path.to_str().unwrap()])
        .env_remove("CLI_BASE")
        .env_remove("CLI_VALUE")
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("CLI_BASE=hello\n"));
    assert!(stdout.contains("CLI_VALUE=hello-world\n"));
}

#[test]
fn can_disable_expansion_and_command_substitution() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("test.env");
    std::fs::write(
        &path,
        "CLI_LITERAL=$MISSING\nCLI_COMMAND=$(printf unsafe)\n",
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_dotenv"))
        .args([
            "--file",
            path.to_str().unwrap(),
            "--no-expand",
            "--no-command-substitution",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("CLI_LITERAL=$MISSING\n"));
    assert!(stdout.contains("CLI_COMMAND=$(printf unsafe)\n"));
}

#[cfg(unix)]
#[test]
fn runs_a_child_with_the_expanded_environment() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("test.env");
    std::fs::write(&path, "CLI_BASE=expand\nCLI_VALUE=${CLI_BASE}ed\n").unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_dotenv"))
        .args([
            "--file",
            path.to_str().unwrap(),
            "--",
            "sh",
            "-c",
            "test \"$CLI_VALUE\" = expanded",
        ])
        .env_remove("CLI_BASE")
        .env_remove("CLI_VALUE")
        .status()
        .unwrap();
    assert!(status.success());
}
#[cfg(windows)]
#[test]
fn runs_a_child_with_the_expanded_environment() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("test.env");
    std::fs::write(&path, "CLI_BASE=expand\nCLI_VALUE=${CLI_BASE}ed\n").unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_dotenv"))
        .args([
            "--file",
            path.to_str().unwrap(),
            "--",
            "cmd",
            "/C",
            "if not %CLI_VALUE%==expanded exit /b 1",
        ])
        .env_remove("CLI_BASE")
        .env_remove("CLI_VALUE")
        .status()
        .unwrap();
    assert!(status.success());
}
