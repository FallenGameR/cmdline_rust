use anyhow::{anyhow, Result};
use clap::{arg, Command};
use std::io::{BufRead, BufReader};

#[derive(Debug, Clone)]
enum TailValue {
    PositiveZero,
    Number(i64),
}

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: TailValue,
    bytes: Option<TailValue>,
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
            arg!(-l --lines <LINES> "From what line to read, e.g. 1 or -1 means print the last one, +1 all but the first one").default_value("10")
                .value_parser(parse_tail_value)
                .conflicts_with_all(["bytes"]),
            arg!(-b --bytes <BYTES> "From what byte to read, e.g. 1 or -1 means print the last one, +1 all but the first one")
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

fn parse_tail_value(_: &str) -> Result<TailValue> {
    todo!("parse_tail_value")
}

pub fn run(config: Config) -> Result<()> {
    dbg!(&config);

    Ok(())
}


fn open(path: &str) -> Result<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(
            std::fs::File::open(path).map_err(|e| anyhow!("{path}: {e}"))?,
        ))),
    }
}





// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{
        count_lines_bytes, get_start_index, parse_num, TailValue::*,
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
        assert_eq!(get_start_index(&PositiveZero, 0), None);

        // +0 from a nonempty file returns an index that
        // is one less than the number of lines/bytes
        assert_eq!(get_start_index(&PositiveZero, 1), Some(0));

        // Taking 0 lines/bytes returns None
        assert_eq!(get_start_index(&Number(0), 1), None);

        // Taking any lines/bytes from an empty file returns None
        assert_eq!(get_start_index(&Number(1), 0), None);

        // Taking more lines/bytes than is available returns None
        assert_eq!(get_start_index(&Number(2), 1), None);

        // When starting line/byte is less than total lines/bytes,
        // return one less than starting number
        assert_eq!(get_start_index(&Number(1), 10), Some(0));
        assert_eq!(get_start_index(&Number(2), 10), Some(1));
        assert_eq!(get_start_index(&Number(3), 10), Some(2));

        // When starting line/byte is negative and less than total,
        // return total - start
        assert_eq!(get_start_index(&Number(-1), 10), Some(9));
        assert_eq!(get_start_index(&Number(-2), 10), Some(8));
        assert_eq!(get_start_index(&Number(-3), 10), Some(7));

        // When the starting line/byte is negative and more than the total,
        // return 0 to print the whole file
        assert_eq!(get_start_index(&Number(-20), 10), Some(0));
    }

    #[test]
    fn test_parse_num() {
        // All integers should be interpreted as negative numbers
        let res = parse_num("3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Number(-3));

        // A leading "+" should result in a positive number
        let res = parse_num("+3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Number(3));

        // An explicit "-" value should result in a negative number
        let res = parse_num("-3");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Number(-3));

        // Zero is zero
        let res = parse_num("0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Number(0));

        // Plus zero is special
        let res = parse_num("+0");
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), PositiveZero);

        // Test boundaries
        let res = parse_num(&i64::MAX.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Number(i64::MIN + 1));

        let res = parse_num(&(i64::MIN + 1).to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Number(i64::MIN + 1));

        let res = parse_num(&format!("+{}", i64::MAX));
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Number(i64::MAX));

        let res = parse_num(&i64::MIN.to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), Number(i64::MIN));

        // A floating-point value is invalid
        let res = parse_num("3.14");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "3.14");

        // Any non-integer string is invalid
        let res = parse_num("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "foo");
    }
}
