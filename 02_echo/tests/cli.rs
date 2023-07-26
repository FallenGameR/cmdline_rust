use assert_cmd::Command;

#[test]
fn dies_no_args() {
    let mut cmd = Command::cargo_bin("echo").unwrap();
    cmd.assert().failure().stderr(predicates::str::contains("Usage:"));
}

#[test]
fn runs() {
    let mut cmd = Command::cargo_bin("echo").unwrap();
    cmd.arg("Hello").assert().success();
}