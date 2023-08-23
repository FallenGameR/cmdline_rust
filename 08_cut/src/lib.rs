use anyhow::{bail, Result};
use clap::{arg, Command};
use std::{
    io::{BufRead, BufReader},
    ops::{Range, RangeInclusive},
};

const PAGE_SIZE: usize = 4096;
const BUFFER_SIZE: usize = PAGE_SIZE * 2;

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
                .conflicts_with("chars")
                .conflicts_with("fields"),
            arg!(-c --chars <CHARS> "What char ranges to extract, e.g. 3-5, 2")
                .value_parser(parse_ranges)
                .conflicts_with("bytes")
                .conflicts_with("fields"),
            arg!(-f --fields <FIELDS> "What field ranges to extract, e.g. 1, 3-5, 2")
                .value_parser(parse_ranges)
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

fn parse_ranges(range: &str) -> Result<Vec<RangeInclusive<usize>>> {
    range.split(',').map(|x| parse_range(x.trim())).collect()
}

fn parse_range(range: &str) -> Result<RangeInclusive<usize>> {
    let result = range
        .split('-')
        .map(|x| x.parse())
        .collect::<Result<Vec<usize>, _>>();

    // Input: inclusive ranges, positive indexes
    // Output: inclusive ranges, zero-based indexes
    let construct = |start, end| -> Result<RangeInclusive<usize>> {
        if start == 0 || end == 0 {
            bail!("Positions start at one, zero is invalid value");
        }

        Ok(start - 1..=end)
    };

    if let Ok(res) = result {
        if res.len() == 1 {
            return construct(res[0], res[0]);
        }

        if res.len() == 2 {
            return construct(res[0], res[1]);
        }
    }

    bail!("Invalid range '{}'", range)
}

pub fn run(config: Config) -> Result<()> {
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

fn open(path: &str) -> Result<Box<dyn BufRead>> {
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

#[cfg(test)]
mod unit_tests {
    //extract_bytes, extract_chars, extract_fields,
    use super::parse_ranges;
    //use csv::StringRecord;

    #[test]
    fn test_parse_ranges() {
        // The empty string is an error
        assert!(parse_ranges("").is_err());

        // Zero is an error
        let res = parse_ranges("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Positions start at one, zero is invalid value",
        );

        let res = parse_ranges("0-1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Positions start at one, zero is invalid value",
        );

        let res = parse_ranges("10-0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "Positions start at one, zero is invalid value",
        );

        // Any non-number is an error
        let res = parse_ranges("a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid range 'a'",);

        let res = parse_ranges("1,a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid range 'a'",);

        let res = parse_ranges("1-a");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid range '1-a'",);

        let res = parse_ranges("a-1");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "Invalid range 'a-1'",);

        // Wonky ranges
        let res = parse_ranges("-");
        assert!(res.is_err());

        let res = parse_ranges(",");
        assert!(res.is_err());

        let res = parse_ranges("1,");
        assert!(res.is_err());

        let res = parse_ranges("1-");
        assert!(res.is_err());

        let res = parse_ranges("1-1-1");
        assert!(res.is_err());

        let res = parse_ranges("1-1-a");
        assert!(res.is_err());

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
        assert_eq!(res.unwrap(), vec![0..=2]);

        let res = parse_ranges("1-+2");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), vec![0..=1]);
    }

    /*
    #[test]
    fn test_extract_fields() {
        let rec = StringRecord::from(vec!["Captain", "Sham", "12345"]);
        assert_eq!(extract_fields(&rec, &[0..1]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2]), &["Sham"]);
        assert_eq!(
            extract_fields(&rec, &[0..1, 2..3]),
            &["Captain", "12345"]
        );
        assert_eq!(extract_fields(&rec, &[0..1, 3..4]), &["Captain"]);
        assert_eq!(extract_fields(&rec, &[1..2, 0..1]), &["Sham", "Captain"]);
    }

    #[test]
    fn test_extract_chars() {
        assert_eq!(extract_chars("", &[0..1]), "".to_string());
        assert_eq!(extract_chars("ábc", &[0..1]), "á".to_string());
        assert_eq!(extract_chars("ábc", &[0..1, 2..3]), "ác".to_string());
        assert_eq!(extract_chars("ábc", &[0..3]), "ábc".to_string());
        assert_eq!(extract_chars("ábc", &[2..3, 1..2]), "cb".to_string());
        assert_eq!(
            extract_chars("ábc", &[0..1, 1..2, 4..5]),
            "áb".to_string()
        );
    }

    #[test]
    fn test_extract_bytes() {
        assert_eq!(extract_bytes("ábc", &[0..1]), "�".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2]), "á".to_string());
        assert_eq!(extract_bytes("ábc", &[0..3]), "áb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..4]), "ábc".to_string());
        assert_eq!(extract_bytes("ábc", &[3..4, 2..3]), "cb".to_string());
        assert_eq!(extract_bytes("ábc", &[0..2, 5..6]), "á".to_string());
    }
    */
}
