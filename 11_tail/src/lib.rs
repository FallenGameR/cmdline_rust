use anyhow::Result;
use clap::{arg, Command};
use std::{fs::File, io::{BufReader, Read, BufRead}, ops::RangeInclusive};

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
    dbg!(&config);

    for file in config.files {
        let file = File::open(file).map_err(anyhow::Error::from)?;
        dbg!(file);
    }

    Ok(())
}

const PAGE_SIZE: usize = 4096;
const BUFFER_SIZE: usize = PAGE_SIZE * 2;

fn count_lines_bytes(path: &str) -> Result<(usize, usize)> {
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

    Ok((lines, bytes))
}

fn get_tail_range(position: &Position, total: usize) -> Option<RangeInclusive<usize>> {
    match position {
        Position::FromHead(n) => {
            Some(*n..=total)
        },
        Position::FromTail(n) => {
            Some(total-n..=total)
        }
    }
}

fn get_start_index(position: &Position, total_lines: usize) -> Option<usize> {
    todo!("get_start_index")
}

fn print_lines(mut file: impl BufRead, position: &Position, total_lines: usize) -> Result<()> {
    todo!("print_lines")
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{
        count_lines_bytes, get_start_index, parse_tail_value, Position::*,
    };

    #[test]
    fn test_count_lines_bytes() {
        let res = count_lines_bytes("tests/inputs/one.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (1, 24));

        let res = count_lines_bytes("tests/inputs/ten.txt");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), (10, 49));
    }

    #[test]
    fn test_get_start_index() {
        // +0 from an empty file (0 lines/bytes) returns None
        assert_eq!(get_start_index(&FromHead(0), 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&FromHead(0), 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&FromTail(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&FromTail(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&FromTail(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&FromTail(1), 10), Some(0));
        assert_eq!(get_start_index(&FromTail(2), 10), Some(1));
        assert_eq!(get_start_index(&FromTail(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&FromTail(1), 10), Some(9));
        assert_eq!(get_start_index(&FromTail(2), 10), Some(8));
        assert_eq!(get_start_index(&FromTail(3), 10), Some(7));

        // When the starting line/byte is negative and more than the total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&FromTail(20), 10), Some(0));
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
