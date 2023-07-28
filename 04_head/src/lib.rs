use std::{error::Error, io::{BufRead, BufReader}};
use clap::{arg, Command};

type DynErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: usize,
    bytes: Option<usize>,
}

pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("head")
        .version("1.0")
        .author("FallenGameR")
        .about("Prints begining of files for a preview")
        .args([
            arg!([files] ... "Files to preview, stdin is -")
                .default_value("-"),
            arg!(-l --lines <line_count> "Number of lines to show")
                .value_parser(clap::value_parser!(usize))
                .default_value("10"),
            arg!(-b --bytes <byte_count> "Number of bytes to show")
                .value_parser(clap::value_parser!(usize)),
        ])
        .get_matches();

    Ok(Config {
        files: matches
            .remove_many("files")
            .expect("No file paths provided")
            .collect(),
        lines: matches
            .remove_one("lines")
            .expect("No number of lines provided"),
        bytes: matches
            .remove_one("bytes"),
    })
}

// cargo run -- -n (ls .\tests\inputs\*.txt)
// cargo run -- -n (walker .\tests\inputs\ -a)
pub fn run(config: Config) -> DynErrorResult<()> {
    println!("{:?}", config);

    for path in &config.files {
        match open(&path) {
            Ok(reader) => process(reader, &config),
            Err(error) => eprintln!("Can't open file {}, error {}", path, error),
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
}