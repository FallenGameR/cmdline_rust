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
    run(&vec!["Hello there"], "tests/expected/hello1.txt")
}

#[test]
fn hello2() -> TestResult {
    run(&vec!["Hello", "there"], "tests/expected/hello2.txt")
}

#[test]
fn hello1_no_newline() -> TestResult {
    run(&vec!["Hello   there", "-n"], "tests/expected/hello1.n.txt")
}

#[test]
fn hello2_no_newline() -> TestResult {
    run(&vec!["-n", "Hello", "there"], "tests/expected/hello2.n.txt")
}

fn run(args: &[&str], path: &str) -> TestResult {
    let expected = fs::read_to_string(path)?;
    let mut cmd = Command::cargo_bin("echo")?;
    cmd.args(args).assert().success().stdout(expected);
    Ok(())
}