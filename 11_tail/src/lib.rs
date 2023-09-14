use anyhow::{Result, anyhow};
use clap::{arg, Command};
use std::{fs::File, io::{BufReader, Read, BufRead}, ops::Range};

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: Position,
    bytes: Option<Position>,
    quiet: bool,
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
                .value_parser(parse_tail_value)
                .conflicts_with_all(["bytes"]),
            arg!(-c --bytes <BYTES> "From what byte to read, e.g. 1 or -1 means print the last one, +1 all but the first one")
                .value_parser(parse_tail_value)
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

#[derive(Debug, Clone, PartialEq)]
enum Position {
    FromTail(usize), //  1, -1, 0, -0
    FromHead(usize), // +1, +0
}

fn parse_tail_value(text: &str) -> Result<Position> {
    match text.parse::<i64>() {
        Ok(value) if text.starts_with('+') => Ok(Position::FromHead(value.try_into()?)),
        Ok(value) if value < 0 => Ok(Position::FromTail((-value).try_into()?)),
        Ok(value) => Ok(Position::FromTail(value.try_into()?)),
        Err(error) => Err(error.into()),
    }
}

pub fn run(config: Config) -> Result<()> {
    for file in config.files {
        let total = count_lines_bytes(&file)?;
        match config.bytes.as_ref() {
            Some(bytes) => print_bytes(file, &bytes, total.bytes)?,
            None => print_lines(file, &config.lines, total.lines)?,
        }
    }

    Ok(())
}

const PAGE_SIZE: usize = 4096;
const BUFFER_SIZE: usize = PAGE_SIZE * 2;

#[derive(Debug, PartialEq)]
struct Total {
    lines: usize,
    bytes: usize,
}

fn count_lines_bytes(path: &str) -> Result<Total> {
    let file = File::open(path).map_err(anyhow::Error::from)?;
    let mut buffer = [0; BUFFER_SIZE];
    let mut reader = BufReader::new(file);

    let mut lines = 0;
    let mut bytes = 0;

    loop {
        let len = reader.read(&mut buffer)?;
        if len == 0 {
            break;
        }

        lines += buffer[0..len].iter().filter(|&&b| b == b'\n').count();
        bytes += len;
    }

    Ok(Total{ lines, bytes })
}

// indexes  01234
// total    5
// position uses 0..=5 and it offset from end or begining
// head     0..=4 from what index to start till the end, e.g. 1 results in 1..5
// tail     1..=5 how many elements to show from the end, e.g. 1 results in 4..5
// in case when position is counted from tail and the range is going to
// be more then full file we return range that covers the whole file
fn get_tail_range(position: &Position, total: usize) -> Option<Range<usize>> {
    let offset = match position {
        Position::FromHead(offset) => *offset,
        Position::FromTail(elements) => total.saturating_sub(*elements),
    };

    if offset >= total { None } else { Some(offset..total) }
}

fn print_lines(file: String, position: &Position, total_lines: usize) -> Result<()> {
    let Some(range) = get_tail_range(position, total_lines) else {
        return Err(anyhow!("{position:?}: invalid line position for file {file}"));
    };

    let lines = BufReader::new(File::open(file)?).lines();
    for line in lines.skip(range.start).take(range.end - range.start) {
        println!("{}", line?);
    }

    Ok(())
}

fn print_bytes(file: String, position: &Position, total_bytes: usize) -> Result<()> {
    let Some(range) = get_tail_range(position, total_bytes) else {
        return Err(anyhow!("{position:?}: invalid byte position for file {file}"));
    };

    let bytes = BufReader::new(File::open(file)?)
        .bytes()
        .skip(range.start)
        .take(range.end - range.start)
        .collect::<Result<Vec<u8>, std::io::Error>>()?;
    print!("{}", String::from_utf8_lossy(&bytes));

    Ok(())
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::Total;

    use super::{
        count_lines_bytes, get_tail_range, parse_tail_value, Position::*,
    };

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Total{ lines: 1, bytes: 24 });

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Total{ lines: 10, bytes: 49 });
    }

    #[test]
    fn test_get_tail_range() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_tail_range(&FromHead(0), 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_tail_range(&FromHead(0), 1), Some(0..1));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_tail_range(&FromTail(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_tail_range(&FromTail(1), 0), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_tail_range(&FromTail(1), 10), Some(9..10));
        assert_eq!(get_tail_range(&FromTail(2), 10), Some(8..10));
        assert_eq!(get_tail_range(&FromTail(3), 10), Some(7..10));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_tail_range(&FromHead(1), 10), Some(1..10));
        assert_eq!(get_tail_range(&FromHead(2), 10), Some(2..10));
        assert_eq!(get_tail_range(&FromHead(3), 10), Some(3..10));

        // When the starting line/byte is negative and more than the total,
        // return 0 to print the whole file
        assert_eq!(get_tail_range(&FromTail(2), 1), Some(0..1));
        assert_eq!(get_tail_range(&FromTail(20), 10), Some(0..10));
    }

    #[test]
    fn test_parse_tail_value() {
        // All integers should be interpreted as negative numbers
        let res = parse_tail_value("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromTail(3));

        // A leading "+" should result in a positive number
        let res = parse_tail_value("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromHead(3));

        // An explicit "-" value should result in a negative number
        let res = parse_tail_value("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromTail(3));

        // Zero is zero
        let res = parse_tail_value("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromTail(0));

        // Plus zero is special
        let res = parse_tail_value("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), FromHead(0));

        // A floating-point value is invalid
        let res = parse_tail_value("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid digit found in string");

        // Any non-integer string is invalid
        let res = parse_tail_value("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid digit found in string");
    }
}
