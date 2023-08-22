use clap::{
    arg,
    error::{ContextKind, ContextValue, ErrorKind},
    Command,
};
use std::{
    error::Error,
    io::{BufRead, BufReader},
    ops::Range,
};

const PAGE_SIZE: usize = 4096;
const BUFFER_SIZE: usize = PAGE_SIZE * 2;

type DynErrorResult<T> = Result<T, Box<dyn Error>>;
type Positions = Vec<Range<usize>>;

#[derive(Debug)]
pub enum ExtractedRanges {
    Bytes(Positions),
    Chars(Positions),
    Fields(Positions),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    extracted: ExtractedRanges,
    delimeter: char,
}

pub fn get_args() -> DynErrorResult<Config> {
    // CLI arguments
    let mut matches = Command::new("cut")
        .version("1.0")
        .author("FallenGameR")
        .about("Extracts data from a text file")
        .args([
            arg!([FILES] ... "Files to process, stdin is -").default_value("-"),
            arg!(-b --bytes <BYTES> ... "What byte ranges to extract, e.g. 1, 3-5")
                .value_parser(parse_range)
                .conflicts_with("chars")
                .conflicts_with("fields"),
            arg!(-c --chars <CHARS> ... "What char ranges to extract, e.g. 3-5, 2")
                .value_parser(parse_range)
                .conflicts_with("bytes")
                .conflicts_with("fields"),
            arg!(-f --fields <FIELDS> ... "What field ranges to extract, e.g. 1, 3-5, 2")
                .value_parser(parse_range)
                .conflicts_with("bytes")
                .conflicts_with("chars"),
            arg!(-d --delimeter <DELIMETER> "Fields delimeter, tab is default")
                .value_parser(clap::value_parser!(char))
                .default_value("\t"),
        ])
        .get_matches();

    // Make sure that only one range type is provided
    let mut selected_count = 0;
    let mut selected = "";

    if matches.contains_id("bytes") {
        selected_count += 1;
        selected = "bytes";
    }

    if matches.contains_id("chars") {
        selected_count += 1;
        selected = "chars";
    }

    if matches.contains_id("fields") {
        selected_count += 1;
        selected = "fields";
    }

    if selected_count != 1 {
        return Err("Please provide either --bytes --chars or --fields once".into());
    }

    // Composing the config
    Ok(Config {
        files: matches
            .remove_many("FILES")
            .expect("No file paths provided")
            .collect(),
        extracted: {
            let ranges = matches
                .remove_many(selected)
                .expect("No ranges were provided")
                .collect();

            match selected {
                "bytes" => ExtractedRanges::Bytes(ranges),
                "chars" => ExtractedRanges::Chars(ranges),
                "fields" => ExtractedRanges::Fields(ranges),
                _ => unreachable!("Unknown range type"),
            }
        },
        delimeter: matches
            .remove_one("delimeter")
            .expect("No delimeter was provided"),
    })
}

fn parse_range(range: &str) -> Result<Range<usize>, clap::Error> {
    let result: Result<Vec<usize>, _> = range
        .split('-')
        .map(|x| x.parse())
        .collect();

    if let Ok(res) = result {
        if res.len() == 1 {
            return Ok(res[0]..res[0] + 1);
        }

        if res.len() == 2 {
            return Ok(res[0]..res[1] + 1);
        }
    }

    let mut clap_error = clap::Error::new(ErrorKind::InvalidValue);
    clap_error.insert(
        ContextKind::InvalidValue,
        ContextValue::String(format!("Invalid range '{}'", range)),
    );
    Err(clap_error)
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
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(path)?))),
    }
}

/*
fn process_file(path: &str, mut reader: Box<dyn BufRead>, config: &Config) {
    unimplemented!()
    let buffer_size = config.bytes.unwrap_or(BUFFER_SIZE);
    let mut buffer = vec![0; buffer_size];

    // Alternatively we could do the following:
    // reader.bytes().take(buffer_size as u64);

    match reader.read(&mut buffer) {
        Ok(len) => process_buffer(&buffer[0..len], config),
        Err(error) => eprintln!("Can't open file '{}', error {}", path, error),
    }
}

fn process_buffer(buffer: &[u8], config: &Config) {
    unimplemented!()
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
}
*/
