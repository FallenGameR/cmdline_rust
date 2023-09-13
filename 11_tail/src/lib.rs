use anyhow::{anyhow, bail, Result};
use clap::{arg, Arg, ArgAction, Command};
use std::cmp::Ordering::{Equal, Greater, Less};
use std::io::{BufRead, BufReader};

#[derive(Debug)]
enum TailValue {
    PositiveZero,
    Number(i64),
}

#[derive(Debug)]
pub struct Config {
    file1: String,
    file2: String,
    show_col1: bool,
    show_col2: bool,
    show_col3: bool,
    case_insensitive: bool,
    delimeter: String,
}

fn output(config: &Config, column: u8, value: &str) {
    // Check if we even need to print this column
    match column {
        1 if !config.show_col1 => return,
        2 if !config.show_col2 => return,
        3 if !config.show_col3 => return,
        _ => (),
    }

    // How many delimeters do we need to print before the value
    let number_of_delimeters = match column {
        1 => 0,
        2 => u8::from(config.show_col1),
        3 => u8::from(config.show_col1) + u8::from(config.show_col2),
        _ => panic!("Invalid column number"),
    };

    // Print value in the corresponding column
    let delimeters = config.delimeter.repeat(number_of_delimeters as usize);
    println!("{delimeters}{value}");
}

pub fn run(config: Config) -> Result<()> {
    // Case insensitivity
    let casing = |entry: std::result::Result<String, std::io::Error>| -> Option<String> {
        match entry {
            Err(error) => { eprintln!("Error: {error}"); None },
            Ok(line) => Some(if config.case_insensitive { line.to_ascii_lowercase() } else { line })
        }
    };

    // Open iterators
    let mut file1 = open(&config.file1)?.lines().filter_map(casing);
    let mut file2 = open(&config.file2)?.lines().filter_map(casing);
    let mut a = file1.next();
    let mut b = file2.next();

    // Line comparison
    loop {
        match (&a, &b) {
            (None, None) => break,
            (Some(a_text), None) => {
                output(&config, 1, a_text);
                a = file1.next();
            },
            (None, Some(b_text)) => {
                output(&config, 2, b_text);
                b = file2.next();
            },
            (Some(a_text), Some(b_text)) => match a_text.cmp(b_text) {
                Equal => {
                    output(&config, 3, a_text);
                    a = file1.next();
                    b = file2.next();
                }
                Less => {
                    output(&config, 1, a_text);
                    a = file1.next();
                }
                Greater => {
                    output(&config, 2, b_text);
                    b = file2.next();
                }
            },
        }
    }

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

pub fn get_args() -> Result<Config> {
    // CLI arguments
    let mut matches = Command::new("grep")
        .version("1.0")
        .author("FallenGameR")
        .about("Finds common and unique lines in two sorted files")
        .args([
            arg!(<FILE1> "First file to process, stdin is -"),
            arg!(<FILE2> "Second file to process, stdin is -"),
            Arg::new("column1")
                .short('1')
                .help("Don't print column1 (unique lines from first file)")
                .action(ArgAction::SetTrue),
            Arg::new("column2")
                .short('2')
                .help("Don't print column2 (unique lines from second file)")
                .action(ArgAction::SetTrue),
            Arg::new("column3")
                .short('3')
                .help("Don't print column3 (common lines in both files)")
                .action(ArgAction::SetTrue),
            arg!(-i --insensitive "Perform case insensitive matching"),
            arg!(-d --delimeter <DELIMETER> "Delimiter to use for columns").default_value("\t"),
        ])
        .get_matches();

    // Check that we don't have both files set to stdin
    let file1: String = matches.remove_one("FILE1").expect("No first file provided");
    let file2: String = matches
        .remove_one("FILE2")
        .expect("No second file provided");
    if file1 == "-" && file2 == "-" {
        bail!("Both input files cannot be STDIN (\"-\")");
    }

    // Construct config
    Ok(Config {
        file1,
        file2,
        show_col1: !matches.get_flag("column1"),
        show_col2: !matches.get_flag("column2"),
        show_col3: !matches.get_flag("column3"),
        case_insensitive: matches.get_flag("insensitive"),
        delimeter: matches
            .remove_one("delimeter")
            .expect("No delimeter was provided"),
    })
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
