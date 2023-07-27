use std::{error::Error, io::{BufRead, BufReader}};

use clap::{arg, Command};

type DynErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

// cargo run -- -n (ls .\tests\inputs\*.txt)
// cargo run -- -n (walker .\tests\inputs\)
// cargo run -- -n (walker .\tests\inputs\ -a)
pub fn run(config: Config) -> DynErrorResult<()> {
    for path in &config.files {
        match open(&path) {
            Ok(reader) => process(reader, &config),
            Err(error) => eprintln!("Can't open file {}, error {}", path, error),
        }
    }
    Ok(())
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
            print!("{:>6}     ", index);
        } else if config.number_nonblank_lines && !buf.trim().is_empty() {
            index += 1;
            print!("{:>6}     ", index);
        }

        print!("{}", buf);
        buf.clear();
    }
}

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

fn open(path: &str) -> DynErrorResult<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(path)?)))
    }
}
