use std::{error::Error, io::{BufRead, BufReader}};
use clap::{arg, Command};

const PAGE_SIZE: usize = 4096;
const BUFFER_SIZE: usize = PAGE_SIZE * 2;

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
            arg!([FILES] ... "Files to preview, stdin is -")
                .default_value("-"),
            arg!(-n --lines <LINES> "Number of lines to show")
                .value_parser(clap::value_parser!(usize))
                .default_value("10")
                .conflicts_with("bytes"),
            arg!(-c --bytes <BYTES> "Number of bytes to show")
                .value_parser(clap::value_parser!(usize))
                .conflicts_with("lines"),
        ])
        .get_matches();

    Ok(Config {
        files: matches
            .remove_many("FILES")
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
    let mutifile_handling = config.files.len() > 1;

    for path in &config.files {
        if mutifile_handling {
            println!("==> {} <==", path);
        }

        match open(&path) {
            Ok(reader) => process_file(&path, reader, &config),
            Err(error) => eprintln!("Can't open file '{}', error {}", &path, error),
        }

        if mutifile_handling {
            println!();
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

fn process_file(path: &str, mut reader: Box<dyn BufRead>, config: &Config) {
    let buffer_size = config.bytes.unwrap_or(BUFFER_SIZE);
    let mut buffer = vec![0; buffer_size];

    match reader.read(&mut buffer) {
        Ok(_) => process_buffer(&buffer, config),
        Err(error) => eprintln!("Can't open file '{}', error {}", path, error),
    }
}

fn process_buffer(buffer: &[u8], config: &Config) {
    let text = String::from_utf8_lossy(buffer);
    let mut line_count = 0;

    for char in text.chars() {
        if char == '\0' {
            break;
        }

        print!("{}", char);

        if char == '\n' {
            line_count += 1;
        }

        if line_count >= config.lines {
            break;
        }
    }
}