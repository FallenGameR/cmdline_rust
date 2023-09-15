use anyhow::Result;
use clap::{arg, Command};
use std::{io::{BufReader, Read, Write}, fs::File};

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: Position,
    bytes: Option<Position>,
    quiet: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Position {
    FromTail(usize), //  1, -1, 0, -0 are valid inputs and represent indexes from the end
    FromHead(usize), // +2, +1, +0 are valid inputs but we have to substract one before storing them to store is as index from start
}

#[derive(Debug)]
enum Total {
    Bytes(usize),
    Lines(usize),
}

pub fn get_args() -> Result<Config> {
    // CLI arguments
    let mut matches = Command::new("tail")
        .version("1.0")
        .author("FallenGameR")
        .about("Previews text from the back of a number of files")
        .args([
            arg!(<FILES> ... "Files to process"),
            arg!(-n --lines <LINES> "From what line to read, e.g. 1 or -1 means print the last one, +1 all but the first one").default_value("10")
                .value_parser(parse_position)
                .conflicts_with_all(["bytes"]),
            arg!(-c --bytes <BYTES> "From what byte to read, e.g. 1 or -1 means print the last one, +1 all but the first one")
                .value_parser(parse_position)
                .conflicts_with_all(["lines"]),
            arg!(-q --quiet "When printing multiple files, don't print the header with file names"),
        ])
        .get_matches();

    // Construct config
    Ok(Config {
        files: matches.remove_many("FILES").expect("At least one file must be provided").collect(),
        lines: matches.remove_one("lines").expect("Default value is provided"),
        bytes: matches.remove_one("bytes"),
        quiet: matches.get_flag("quiet"),
    })
}

fn parse_position(text: &str) -> Result<Position> {
    match text.parse::<i64>() {
        Ok(value) if text.starts_with('+') => Ok(Position::FromHead(TryInto::<usize>::try_into(value)?.saturating_sub(1))),
        Ok(value) if value < 0 => Ok(Position::FromTail((-value).try_into()?)),
        Ok(value) => Ok(Position::FromTail(value.try_into()?)),
        Err(error) => Err(error.into()),
    }
}

pub fn run(config: Config) -> Result<()> {
    let is_multifile = config.files.len() > 1 && !config.quiet;
    let mut files_processed = 0;

    for file in &config.files {
        if is_multifile {
            println!("==> {file} <==");
        }

        match config.bytes.as_ref() {
            Some(bytes) => print_tail(file, &bytes, Total::Bytes(count_bytes(&file)?))?,
            None => print_tail(file, &config.lines, Total::Lines(count_lines(&file)?))?,
        }

        files_processed += 1;
        let is_last_file = files_processed == config.files.len();
        if is_multifile && !is_last_file {
            println!();
        }
    }

    Ok(())
}

// To make it faster we need to read from the end of the file and use IoSlice for output
// Or use File::seek =)
fn print_tail(file: &str, position: &Position, total: Total) -> Result<()> {
    // Variables that are different for printing the tail for line or bytes
    let (size, name, filter): (_,_, &dyn Fn(u8) -> bool) = match total {
        Total::Bytes(bytes) => (bytes, "byte", &|_| true),
        Total::Lines(lines) => (lines, "line", &|b| b == b'\n'),
    };

    // Print error for invalid positions but don't terminate the program
    let Some(offset) = get_offset(position, size) else {
        eprintln!("{position:?}: invalid {name} position for file {file}");
        return Ok(());
    };

    // Rewinding the byte streem to the needed position and take till the end
    let mut skipped = 0;
    let bytes = BufReader::new(File::open(file)?)
        .bytes()
        .filter_map(Result::ok)
        .skip_while(|&b| {
            if skipped == offset {
                return false;
            }

            if filter(b) {
                skipped += 1;
            }

            return true;
        })
        .collect::<Vec<u8>>();

    // Output the result to stdout
    let mut stdout = std::io::stdout();
    stdout.write_all(bytes.as_slice())?;
    stdout.flush()?;

    Ok(())
}

fn count_bytes(path: &str) -> Result<usize> {
    Ok(std::fs::metadata(path)?.len().try_into()?)
}

// TODO: is it fast? if yes we can find byte offset of a needed line
// is it faster to read via an explict buffer or this version is bufferen already?
fn count_lines(path: &str) -> Result<usize> {
    let lines = File::open(path)?
        .bytes()
        .filter_map(Result::ok)
        .fold(0, |a, c| a + (c == b'\n') as usize);

    Ok(lines)
}

// indexes  01234
// total    5
// position uses 0..=5 and it offset from end or begining
// head     0..=4 from what index to start till the end, e.g. 1 results in 1..5
// tail     1..=5 how many elements to show from the end, e.g. 1 results in 4..5
// in case when position is counted from tail and the range is going to
// be more then full file we return range that covers the whole file
fn get_offset(position: &Position, total: usize) -> Option<usize> {
    let offset = match position {
        Position::FromHead(offset) => *offset,
        Position::FromTail(elements) => total.saturating_sub(*elements),
    };

    if offset >= total { None } else { Some(offset) }
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::{count_bytes, count_lines};

    use super::{
        get_offset, parse_position, Position::*,
    };

    #[test]
    fn test_count_lines_bytes() {
        let res = count_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 24);

        let res = count_lines("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 1);

        let res = count_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 49);

        let res = count_lines("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 10);
    }

    #[test]
    fn test_get_offset() {
        // These tests are formulated to be backward compatible with the original tail spec
        // They convert from ambigious spec to indexes range

        assert_eq!(get_offset(&FromHead(0), 0), None);
        assert_eq!(get_offset(&FromHead(0), 1), Some(0));

        assert_eq!(get_offset(&FromTail(0), 1), None);
        assert_eq!(get_offset(&FromTail(1), 0), None);

        assert_eq!(get_offset(&FromTail(1), 10), Some(9));
        assert_eq!(get_offset(&FromTail(2), 10), Some(8));
        assert_eq!(get_offset(&FromTail(3), 10), Some(7));

        assert_eq!(get_offset(&FromHead(1), 10), Some(1));
        assert_eq!(get_offset(&FromHead(2), 10), Some(2));
        assert_eq!(get_offset(&FromHead(3), 10), Some(3));

        assert_eq!(get_offset(&FromTail(2), 1), Some(0));
        assert_eq!(get_offset(&FromTail(20), 10), Some(0));
    }

    #[test]
    fn test_parse_tail_value() {
        // These tests are formulated to be backward compatible with the original tail spec
        // They convert from ambigious spec to indexes range

        let res = parse_position("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromTail(3));

        let res = parse_position("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromTail(3));

        let res = parse_position("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromHead(2));

        let res = parse_position("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromTail(0));

        let res = parse_position("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromHead(0));

        let res = parse_position("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid digit found in string");

        let res = parse_position("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid digit found in string");
    }
}
