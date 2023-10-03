use assert_cmd::Command;
use predicates::prelude::*;
use std::{error::Error, fs::File, io::Read};

type TestResult = Result<(), Box<dyn Error>>;

const PRG: &str = "cal";

// --------------------------------------------------
#[test]
fn dies_year_0() -> TestResult {
    Command::cargo_bin(PRG)?
        .arg("0")
        .assert()
        .failure()
        .stderr("year 0 not in the range [99,9999]\n");
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_year_13() -> TestResult {
    Command::cargo_bin(PRG)?
        .arg("10000")
        .assert()
        .failure()
        .stderr("year 10000 not in the range [99,9999]\n");
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_invalid_year() -> TestResult {
    Command::cargo_bin(PRG)?
        .arg("foo")
        .assert()
        .failure()
        .stderr("invalid digit found in string\n");
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_month_0() -> TestResult {
    Command::cargo_bin(PRG)?
        .args(["-m", "0"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("month 0 not in the range [1,12]"));
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_month_13() -> TestResult {
    Command::cargo_bin(PRG)?
        .args(["-m", "13"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("month 13 not in the range [1,12]"));
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_invalid_month() -> TestResult {
    Command::cargo_bin(PRG)?
        .args(["-m", "foo"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("invalid value 'foo' for '--month <MONTH>'"));
    Ok(())
}

// --------------------------------------------------
#[test]
fn dies_y_and_month() -> TestResult {
    let expected = "the argument '--month <MONTH>' cannot be used with '--show_full_year'";
    Command::cargo_bin(PRG)?
        .args(["-m", "1", "-y"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(expected));
    Ok(())
}

// --------------------------------------------------
#[test]
fn month_num() -> TestResult {
    let expected = &[
        ("1", "January"),
        ("2", "February"),
        ("3", "March"),
        ("4", "April"),
        ("5", "May"),
        ("6", "June"),
        ("7", "July"),
        ("8", "August"),
        ("9", "September"),
        ("10", "October"),
        ("11", "November"),
        ("12", "December"),
    ];

    for (num, month) in expected {
        Command::cargo_bin(PRG)?
            .args(["-m", num])
            .assert()
            .success()
            .stdout(predicates::str::contains((*month).to_string()));
    }
    Ok(())
}

// --------------------------------------------------
#[test]
fn partial_month() -> TestResult {
    let expected = &[
        ("ja", "January"),
        ("f", "February"),
        ("mar", "March"),
        ("ap", "April"),
        ("may", "May"),
        ("jun", "June"),
        ("jul", "July"),
        ("au", "August"),
        ("s", "September"),
        ("n", "November"),
        ("d", "December"),
    ];

    for (arg, month) in expected {
        Command::cargo_bin(PRG)?
            .args(["-m", arg])
            .assert()
            .success()
            .stdout(predicates::str::contains((*month).to_string()));
    }
    Ok(())
}

// --------------------------------------------------
fn run(args: &[&str], expected_file: &str) -> TestResult {
    // Extra work here due to lossy UTF
    let mut file = File::open(expected_file)?;
    let mut expected = Vec::new();
    file.read_to_end(&mut expected)?;

    let assert = Command::cargo_bin(PRG)?
        .args(args)
        .assert()
        .success();

    let output = assert.get_output();
    let stdout = &output.stdout;

    dbg!(String::from_utf8_lossy(&expected));
    dbg!(String::from_utf8_lossy(&stdout));
    assert_eq!(&expected, stdout);

    Ok(())
}

// --------------------------------------------------
#[test]
fn default_one_month() -> TestResult {
    let cmd = Command::cargo_bin(PRG)?.arg("-C").assert().success();
    let out = cmd.get_output();
    let stdout = String::from_utf8(out.stdout.clone())?;
    let lines: Vec<_> = stdout.split('\n').collect();
    assert_eq!(lines.len(), 9);
    assert_eq!(lines[0].len(), 22);
    Ok(())
}

// --------------------------------------------------
#[test]
fn test_2_2020_leap_year() -> TestResult {
    run(&["-m", "2", "2020", "-C"], "tests/expected/2-2020.txt")
}

// --------------------------------------------------
#[test]
fn test_4_2020() -> TestResult {
    run(&["-m", "4", "2020", "-C"], "tests/expected/4-2020.txt")
}

// --------------------------------------------------
#[test]
fn test_april_2020() -> TestResult {
    run(&["2020", "-m", "april", "-C"], "tests/expected/4-2020.txt")
}

// --------------------------------------------------
#[test]
fn test_2020() -> TestResult {
    run(&["2020", "-C"], "tests/expected/2020.txt")
}

// --------------------------------------------------
#[test]
fn year() -> TestResult {
    let cmd = Command::cargo_bin(PRG)?.args(["-y", "-C"]).assert().success();
    let stdout = String::from_utf8(cmd.get_output().stdout.clone())?;
    let lines: Vec<&str> = stdout.split('\n').collect();
    assert_eq!(lines.len(), 38);
    Ok(())
}
