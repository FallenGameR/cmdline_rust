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

//#[test]
//fn true_ok() {
//    let mut cmd = Command::
//    cmd.assert().success();
//}

#[test]
fn false_ok() {
    let mut cmd = Command::cargo_bin("false").unwrap();
    cmd.assert().failure();
}