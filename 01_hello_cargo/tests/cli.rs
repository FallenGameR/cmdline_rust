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