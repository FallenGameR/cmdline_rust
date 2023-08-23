use std::{error::Error, io::{BufRead, BufReader}};

use clap::{arg, Command};

type DynErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

/// # Errors
/// Error is returned when there is a problem while parsing the arguments.
pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("cat")
        .version("1.0")
        .author("FallenGameR")
        .about("Concatenates files and optionally add line numbers")
        .args([
            arg!([files] ... "Input files to concatenate, stdin is -").default_value("-"),
            arg!(-n --number_lines "Add line numbers to output")
                .conflicts_with("number_nonblank_lines"),
            arg!(-b --number_nonblank_lines "Number only nonblack lines")
                .conflicts_with("number_lines"),
        ])
        .get_matches();

    Ok(Config {
        files: matches
            .remove_many("files")
            .expect("No file paths provided")
            .collect(),
        number_lines: matches.get_flag("number_lines"),
        number_nonblank_lines: matches.get_flag("number_nonblank_lines"),
    })
}

/// # Errors
/// Error is returned when program could not process files
pub fn run(config: Config) -> DynErrorResult<()> {
    for path in &config.files {
        match open(path) {
            Ok(reader) => process(reader, &config),
            Err(error) => eprintln!("Can't open file {path}, error {error}"),
        }
    }
    Ok(())
}

fn open(path: &str) -> DynErrorResult<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(path)?)))
    }
}

fn process(mut reader: Box<dyn BufRead>, config: &Config) {
    let mut buf = String::new();
    let mut index = 0;

    while let Ok(read_bytes) = reader.read_line(&mut buf) {
        if read_bytes == 0 {
            break;
        }

        if config.number_lines {
            index += 1;
            print!("{index:>6}\t");
        } else if config.number_nonblank_lines && !buf.trim().is_empty() {
            index += 1;
            print!("{index:>6}\t");
        }

        print!("{buf}");
        buf.clear();
    }
}
