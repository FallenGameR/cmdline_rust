use assert_cmd::Command;

#[test]
fn works() {
    assert!(true);
}

#[test]
fn runs() {
    let mut cmd = Command::cargo_bin("hello_cargo").unwrap();
    cmd.assert().success();
}

#[test]
fn ls_ok() {
    let mut cmd = std::process::Command::new("ls");
    let rest = cmd.output();
    assert!(rest.is_ok());
}

