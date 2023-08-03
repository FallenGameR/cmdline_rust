use clap::{arg, Command};
use std::{
    error::Error,
    io::{BufRead, BufReader},
};

type DynErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    lines: bool,
    words: bool,
    bytes: bool,
    chars: bool,
}

#[derive(Debug, PartialEq)]
pub struct Stats {
    lines: usize,
    words: usize,
    bytes: usize,
    chars: usize,
}

pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("head")
        .version("1.0")
        .author("FallenGameR")
        .about("Calculates string statistic from files: lines, words, chars, bytes")
        .args([
            arg!([FILES] ... "Files to process, stdin is -").default_value("-"),
            arg!(-l --lines "Count lines as number of EOL sequences"),
            arg!(-w --words "Count words as strings separated by whitespace"),
            arg!(-c --chars "Count number of characters"),
            arg!(-b --bytes "Count number of bytes"),
        ])
        .get_matches();

    let mut config = Config {
        files: matches
            .remove_many("FILES")
            .expect("No file paths provided")
            .collect(),
        lines: matches.get_flag("lines"),
        words: matches.get_flag("words"),
        chars: matches.get_flag("chars"),
        bytes: matches.get_flag("bytes"),
    };

    // If no flags are provided, default to all flags are on
    if !config.lines && !config.words && !config.chars && !config.bytes {
        config.lines = true;
        config.words = true;
        config.chars = true;
        config.bytes = true;
    }

    Ok(config)
}

// cargo run -- -n (ls .\tests\inputs\*.txt)
// cargo run -- -n (walker .\tests\inputs\ -a)
pub fn run(config: Config) -> DynErrorResult<()> {
    println!("{:?}", config);

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
}

fn open(path: &str) -> DynErrorResult<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(path)?))),
    }
}


fn process_file(path: &str, mut reader: Box<dyn BufRead>, config: &Config) {
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

fn process_stats(reader: impl BufRead) -> DynErrorResult<Stats> {
    let mut result = Stats { lines: 0, words: 0, bytes: 0, chars: 0 };

    for line in reader.lines() {
        let line = line?;

        result.bytes += line.len();
        result.chars += line.chars().count();
        result.words += line.split_whitespace().count();
        result.lines += 1;
    }

    Ok(result)
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{:>8}", value)
    } else {
        "".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{process_stats, format_field, Stats};
    use std::io::Cursor;

    #[test]
    fn test_stats() {
        let text = "I don't want the world. I just want your half.\r\n";
        let stats = process_stats(Cursor::new(text));

        assert!(stats.is_ok());
        let expected = Stats {
            lines: 1,
            words: 10,
            chars: 48,
            bytes: 48,
        };
        assert_eq!(stats.unwrap(), expected);
    }

    #[test]
    fn test_format_field() {
        assert_eq!(format_field(1, false), "");
        assert_eq!(format_field(3, true), "       3");
        assert_eq!(format_field(10, true), "      10");
    }
}
