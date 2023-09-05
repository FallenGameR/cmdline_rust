mod ranges;

use anyhow::{bail, Result};
use clap::{arg, Command};
use csv::StringRecord;
use ranges::ranges_iter;
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

fn process_file(path: &str, reader: Box<dyn BufRead>, config: &Config) -> Result<()> {
    for line in reader.lines() {
        match line {
            Err(error) => eprintln!("Can't read line from file '{path}', error {error}"),
            Ok(line) => {
                let extracted = match &config.extracted {
                    ExtractedRanges::Bytes(ranges) => extract_bytes(&line, ranges),
                    ExtractedRanges::Chars(ranges) => extract_chars(&line, ranges),
                    ExtractedRanges::Fields(ranges) => {
                        extract_fields(&line, config.delimeter, ranges)?
                    }
                };

                println!("{extracted}");
            }
        }
    }

    Ok(())
}

fn extract_chars(line: &str, ranges: &[RangeInclusive<usize>]) -> String {
    ranges_iter(ranges)
        .filter_map(|i| line.chars().nth(i))
        .collect()
}

fn extract_bytes(line: &str, ranges: &[RangeInclusive<usize>]) -> String {
    let bytes: Vec<u8> = ranges_iter(ranges)
        .filter_map(|i| line.as_bytes().get(i).copied())
        .collect();
    String::from_utf8_lossy(&bytes).into()
}

// https://docs.rs/csv/latest/csv/tutorial/index.html
fn extract_fields(line: &str, delimeter: char, ranges: &[RangeInclusive<usize>]) -> Result<String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .delimiter(delimeter as u8)
        .from_reader(line.as_bytes());
    let record = reader.records().next().expect("No fields found")?;

    // Could be inlined, but tests depend on extract_fields_internal
    // let fields: Vec<&str> = ranges_iter(ranges).filter_map(|i| record.get(i)).collect();
    let fields = extract_fields_internal(&record, ranges);

    let mut writer = csv::WriterBuilder::new()
        .has_headers(false)
        .delimiter(delimeter as u8)
        .from_writer(vec![]);
    writer.write_record(fields)?;
    // println! from process_file adds extra newline so we trim it here
    String::from_utf8(writer.into_inner()?)
        .map(|v| v.trim().to_string())
        .map_err(Into::into)
}

fn extract_fields_internal<'rec>(
    record: &'rec StringRecord,
    ranges: &[RangeInclusive<usize>],
) -> Vec<&'rec str> {
    ranges_iter(ranges).filter_map(|i| record.get(i)).collect()
}

#[cfg(test)]
mod unit_tests {
    use csv::StringRecord;

    use super::parse_ranges;
    use crate::{extract_bytes, extract_chars, extract_fields_internal};

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_ranges() {
        // The empty string is an error
        assert!(parse_ranges("").is_err());

        // Zero is an error
        let res = parse_ranges("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '0' - number would be zero for non-zero type",
        );

        let res = parse_ranges("0-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '0-1' - number would be zero for non-zero type",
        );

        let res = parse_ranges("10-0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '10-0' - number would be zero for non-zero type",
        );

        // Any non-number is an error
        let res = parse_ranges("a");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range 'a' - invalid digit found in string"
        );

        let res = parse_ranges("1,a");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range 'a' - invalid digit found in string"
        );

        let res = parse_ranges("1-a");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '1-a' - invalid digit found in string"
        );

        let res = parse_ranges("a-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range 'a-1' - invalid digit found in string"
        );

        // Wonky ranges
        let res = parse_ranges("-");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '-' - cannot parse integer from empty string"
        );

        let res = parse_ranges(",");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '' - cannot parse integer from empty string"
        );

        let res = parse_ranges("1,");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '' - cannot parse integer from empty string"
        );

        let res = parse_ranges("1-");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '1-' - cannot parse integer from empty string"
        );

        let res = parse_ranges("1-1-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '1-1-1' - wrong number of range parts 3"
        );

        let res = parse_ranges("1-1-a");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Invalid range '1-1-a' - invalid digit found in string"
        );

        // All the following are acceptable
        let res = parse_ranges("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=0]);

        let res = parse_ranges("01");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=0]);

        let res = parse_ranges("1,3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=0, 2..=2]);

        let res = parse_ranges("001,0003");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=0, 2..=2]);

        let res = parse_ranges("1-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=2]);

        let res = parse_ranges("0001-03");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=2]);

        let res = parse_ranges("1,7,3-5");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=0, 6..=6, 2..=4]);

        let res = parse_ranges("15,19-20");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![14..=14, 18..=19]);

        let res = parse_ranges("1-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=0]);

        let res = parse_ranges("2-1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![1..=0]);

        // A leading "+" is not an error
        let res = parse_ranges("+1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=0]);

        let res = parse_ranges("+1-2");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=1]);

        let res = parse_ranges("1-+2");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=1]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..=0]), String::new());
        assert_eq!(extract_chars("ábc", &[0..=0]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..=0, 2..=2]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..=2]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..=2, 1..=1]), "cb".to_string());
        assert_eq!(
            extract_chars("ábc", &[0..=0, 1..=1, 4..=4]),
            "áb".to_string()
        );
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..=0]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..=1]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..=2]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..=3]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..=3, 2..=2]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..=1, 5..=5]), "á".to_string());
    }

    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields_internal(&rec, &[0..=0]), &["Captain"]);
        assert_eq!(extract_fields_internal(&rec, &[1..=1]), &["Sham"]);
        assert_eq!(
            extract_fields_internal(&rec, &[0..=0, 2..=2]),
            &["Captain", "12345"]
        );
        assert_eq!(extract_fields_internal(&rec, &[0..=0, 3..=3]), &["Captain"]);
        assert_eq!(
            extract_fields_internal(&rec, &[1..=1, 0..=0]),
            &["Sham", "Captain"]
        );
    }
}
