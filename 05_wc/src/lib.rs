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

impl std::ops::AddAssign for Stats {
    fn add_assign(&mut self, other: Stats) {
        self.lines += other.lines;
        self.words += other.words;
        self.chars += other.chars;
        self.bytes += other.bytes;
    }
}

pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("head")
        .version("1.0")
        .author("FallenGameR")
        .about("Calculates string statistic from files: lines, words, chars, bytes")
        .args([
            arg!([files] ... "Files to process, stdin is -").default_value("-"),
            arg!(-l --lines "Count lines as number of EOL sequences"),
            arg!(-w --words "Count words as strings separated by whitespace"),
            arg!(-c --chars "Count number of characters"),
            arg!(-b --bytes "Count number of bytes"),
        ])
        .get_matches();

    let mut config = Config {
        files: matches
            .remove_many("files")
            .expect("No file paths provided")
            .collect(),
        lines: matches.get_flag("lines"),
        words: matches.get_flag("words"),
        chars: matches.get_flag("chars"),
        bytes: matches.get_flag("bytes"),
    };

    // If no flags are provided, use all flags
    //
    // That is not backward compatible since wc was written in a day when chars and bytes
    // meant the same (and that is reflected in the parameter name -c for byte count).
    // Then Unicode came and suddenly there was a need to distinguish between bytes and chars.
    // -c was occupied and thus -m was added for bytes. For backward compatible reasons
    // 3 output column remained and that made the last column ambiguous.
    //
    // Nowadays UTF-8 is common and if we don't want to be backward compatible and instead
    // make wc program anew we should use -b for bytes and -c for chars and by default output
    // 4 columns to solve the ambiguity.
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
    let mut totals = Stats { lines: 0, words: 0, bytes: 0, chars: 0 };
    let mut files_processed = 0;

    for path in &config.files {
        match open(path) {
            Err(error) => eprintln!("Can't open file '{}', error {}", &path, error),
            Ok(reader) =>
            {
                let stats = process_stats(reader)?;
                output_stats(&stats, path, &config);
                files_processed += 1;
                totals += stats;
            },
        }
    }

    if files_processed > 1 {
        output_stats(&totals, "total", &config);
    }

    Ok(())
}

fn process_stats(mut reader: impl BufRead) -> DynErrorResult<Stats> {
    let mut result = Stats { lines: 0, words: 0, bytes: 0, chars: 0 };
    let mut line = String::new();

    loop {
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        result.bytes += bytes;
        result.chars += line.chars().count();
        result.words += line.split_whitespace().count();
        result.lines += 1;

        line.clear();
    }

    Ok(result)
}

fn output_stats(stats: &Stats, name: &str, config: &Config)
{
    println!(
        "{}{}{}{} {}",
        format_field(stats.lines, config.lines),
        format_field(stats.words, config.words),
        format_field(stats.chars, config.chars),
        format_field(stats.bytes, config.bytes),
        name
    );
}

fn format_field(value: usize, show: bool) -> String {
    if show {
        format!("{value:>8}")
    } else {
        String::new()
    }
}

fn open(path: &str) -> DynErrorResult<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(path)?))),
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
