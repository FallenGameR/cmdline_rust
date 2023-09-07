use anyhow::{bail, Result};
use clap::{arg, Command};
use std::{
    io::{BufRead, BufReader},
    num::NonZeroUsize,
    ops::RangeInclusive,
};

type Positions = Vec<RangeInclusive<usize>>;

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

pub fn run(config: Config) -> Result<()> {
    for path in &config.files {
        match open(path) {
            Ok(reader) => process_file(path, reader, &config)?,
            Err(error) => eprintln!("Can't open file '{}', error {}", &path, error),
        }
    }

    Ok(())
}

fn open(path: &str) -> Result<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(path)?))),
    }
}

pub fn get_args() -> Result<Config> {
    // CLI arguments
    let mut matches = Command::new("cut")
        .version("1.0")
        .author("FallenGameR")
        .about("Extracts data from a text file")
        .args([
            arg!([FILES] ... "Files to process, stdin is -").default_value("-"),
            arg!(-b --bytes <BYTES> "What byte ranges to extract, e.g. 1, 3-5")
                .value_parser(parse_ranges)
                .conflicts_with_all(["chars", "fields"]),
            arg!(-c --chars <CHARS> "What char ranges to extract, e.g. 5-1, 2")
                .value_parser(parse_ranges)
                .conflicts_with_all(["bytes", "fields"]),
            arg!(-f --fields <FIELDS> "What field ranges to extract, e.g. 1, 3")
                .value_parser(parse_ranges)
                .conflicts_with_all(["bytes", "chars"]),
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
        bail!("Please provide either --bytes --chars or --fields once");
    }

    // Composing the config
    Ok(Config {
        files: matches
            .remove_many("FILES")
            .expect("No file paths provided")
            .collect(),
        extracted: {
            let ranges = matches
                .remove_one(selected)
                .expect("No ranges were provided");

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

fn parse_ranges(ranges: &str) -> Result<Vec<RangeInclusive<usize>>> {
    ranges.split(',').map(|x| parse_range(x.trim())).collect()
}

fn parse_range(range: &str) -> Result<RangeInclusive<usize>> {
    let result: Result<Vec<NonZeroUsize>, _> = range.split('-').map(str::parse).collect();

    // Input: inclusive range as indexes, positive indexes
    // Output: inclusive range as range, zero-based indexes
    let construct = |start, end| -> Result<RangeInclusive<usize>> {
        Ok(usize::from(start) - 1..=usize::from(end) - 1)
    };

    match result {
        Err(error) => bail!("Invalid range '{}' - {}", range, error),
        Ok(parts) => match parts.len() {
            1 => construct(parts[0], parts[0]),
            2 => construct(parts[0], parts[1]),
            _ => bail!(
                "Invalid range '{}' - wrong number of range parts {}",
                range,
                parts.len()
            ),
        },
    }
}

fn process_file(path: &str, reader: Box<dyn BufRead>, _: &Config) -> Result<()> {
    for line in reader.lines() {
        match line {
            Err(error) => eprintln!("Can't read line from file '{path}', error {error}"),
            Ok(_) => {
                todo!("Process the line")
            }
        }
    }

    Ok(())
}
