use std::process::Command;

#[cfg(unix)]
#[test]
fn child_exit_codes_and_signals_are_preserved() {
    use std::os::unix::process::ExitStatusExt;
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("empty.env");
    std::fs::write(&path, "").unwrap();
    for (script, code, signal) in [
        ("exit 37", Some(37), None),
        ("kill -TERM $$", None, Some(15)),
    ] {
        let status = Command::new(env!("CARGO_BIN_EXE_dotenv"))
            .arg("-f")
            .arg(&path)
            .args(["--", "sh", "-c", script])
            .status()
            .unwrap();
        assert_eq!(status.code(), code);
        assert_eq!(status.signal(), signal);
    }
}

#[test]
fn layers_files_and_flushes_large_output_without_expansion() {
    let directory = tempfile::tempdir().unwrap();
    let first = directory.path().join("first.env");
    let second = directory.path().join("second.env");
    let mut expected = String::new();
    for index in 0..2000 {
        expected.push_str(&format!("CLI_BUFFER_{index}=value_{index}\n"));
    }
    std::fs::write(&first, &expected).unwrap();
    std::fs::write(&second, "CLI_BUFFER_0=replaced\nCLI_LAST=tail\n").unwrap();
    expected = expected.replacen("CLI_BUFFER_0=value_0", "CLI_BUFFER_0=replaced", 1);
    expected.push_str("CLI_LAST=tail\n");
    let output = Command::new(env!("CARGO_BIN_EXE_dotenv"))
        .args(["--no-expand", "--override", "-f"])
        .arg(&first)
        .arg("-f")
        .arg(&second)
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

#[test]
fn no_expand_respects_inherited_values_unless_overridden() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("literal.env");
    std::fs::write(&path, "CLI_LITERAL=$UNCHANGED\n").unwrap();
    for (override_existing, expected) in [(false, ""), (true, "$UNCHANGED")] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_dotenv"));
        command.args(["--no-expand", "-f"]).arg(&path);
        if override_existing {
            command.arg("--override");
        }
        let output = command.env("CLI_LITERAL", "").output().unwrap();
        assert!(output.status.success());
        assert_eq!(
            output.stdout,
            format!("CLI_LITERAL={expected}\n").as_bytes()
        );
    }
}

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
