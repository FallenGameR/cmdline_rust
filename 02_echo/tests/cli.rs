use assert_cmd::Command;
use std::fs;

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn dies_no_args() -> TestResult {
    let mut cmd = Command::cargo_bin("echo")?;
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Usage:"));
    Ok(())
}

#[test]
fn runs() -> TestResult {
    let mut cmd = Command::cargo_bin("echo")?;
    cmd.arg("Hello").assert().success();
    Ok(())
}

#[test]
fn hello1() -> TestResult {
    let path = "tests/expected/hello1.txt";
    let expected = fs::read_to_string(path)?;
    let mut cmd = Command::cargo_bin("echo")?;
    cmd.arg("Hello there").assert().success().stdout(expected);
    Ok(())
}
