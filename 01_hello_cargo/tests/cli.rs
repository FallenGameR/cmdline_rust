use std::process::Command;

#[test]
fn works() {
    assert!(true);
}

#[test]
fn runs() {
    let mut cmd = Command::new("ls");
    let res = cmd.output();
    assert!(res.is_ok());
}