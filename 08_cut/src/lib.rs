use std::{error::Error, io::{BufRead, BufReader}, ops::Range};
use clap::{arg, Command};

const PAGE_SIZE: usize = 4096;
const BUFFER_SIZE: usize = PAGE_SIZE * 2;

type DynErrorResult<T> = Result<T, Box<dyn Error>>;
type Positions = Vec<Range<usize>>;

#[derive(Debug)]
pub enum ExtractedRanges
{
    Bytes(Positions),
    Chars(Positions),
    Fields(Positions),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    extracted: ExtractedRanges,
    delimeter: u8,              // can be only an ASCII char
}

pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("cut")
        .version("1.0")
        .author("FallenGameR")
        .about("Extracts data from a text file")
        .args([
            arg!([FILES] ... "Files to process, stdin is -")
                .default_value("-"),
            arg!(-b --bytes <BYTES> ... "What byte ranges to extract, e.g. 1, 3-5, 2")
                .value_parser(clap::value_parser!(usize))
                .conflicts_with("chars")
                .conflicts_with("fields"),
            arg!(-c --chars <CHARS> ... "What char ranges to extract, e.g. 1, 3-5, 2")
                .value_parser(clap::value_parser!(usize))
                .conflicts_with("bytes")
                .conflicts_with("fields"),
            arg!(-f --fields <FIELDS> ... "What field ranges to extract, e.g. 1, 3-5, 2")
                .value_parser(clap::value_parser!(usize))
                .conflicts_with("bytes")
                .conflicts_with("chars"),
            arg!(-d --delimeter "Fields delimeter, tab is default")
                .default_value("\t")
                .value_parser(clap::value_parser!(u8)),
        ])
        .get_matches();

    let range_type_count =
        matches.contains_id("bytes") as u8 +
        matches.contains_id("chars") as u8 +
        matches.contains_id("fields") as u8;
    if range_type_count != 1 {
        return Err("Please provide either --bytes --chars or --fields once".into());
    }

    Ok(Config {
        files: matches
            .remove_many("FILES")
            .expect("No file paths provided")
            .collect(),
        lines: matches
            .remove_many("lines")
            .expect("No number of lines provided"),
        delimeter: matches
            .remove_one("delimeter")
            .expect("No delimeter was provided"),
    })
}

fn parse_positions(range: &str) -> DynErrorResult<Positions> {
    unimplemented!()
}

pub fn run(config: Config) -> DynErrorResult<()> {
    println!("{:?}", config);
    Ok(())
    /*
    let mutifile_handling = config.files.len() > 1;
    let mut entry_separator_needed = false;

    for path in &config.files {
        if entry_separator_needed {
            println!();
        }

        entry_separator_needed = true;

        if mutifile_handling {
            println!("==> {} <==", path);
        }

        match open(&path) {
            Ok(reader) => process_file(&path, reader, &config),
            Err(error) => eprintln!("Can't open file '{}', error {}", &path, error),
        }
    }

    Ok(())
    */
}

fn open(path: &str) -> DynErrorResult<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(path)?)))
    }
}

fn process_file(path: &str, mut reader: Box<dyn BufRead>, config: &Config) {
    unimplemented!()
    /*
    let buffer_size = config.bytes.unwrap_or(BUFFER_SIZE);
    let mut buffer = vec![0; buffer_size];

    // Alternatively we could do the following:
    // reader.bytes().take(buffer_size as u64);

    match reader.read(&mut buffer) {
        Ok(len) => process_buffer(&buffer[0..len], config),
        Err(error) => eprintln!("Can't open file '{}', error {}", path, error),
    }
    */
}

fn process_buffer(buffer: &[u8], config: &Config) {
    unimplemented!()
    /*
    let text = String::from_utf8_lossy(buffer);
    let mut line_count = 0;

    for char in text.chars() {
        print!("{}", char);

        if char == '\n' {
            line_count += 1;
        }

        if line_count >= config.lines {
            break;
        }
    }
    */
}