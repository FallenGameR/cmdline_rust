use crate::FileEntityType::*;
use clap::{arg, builder::PossibleValuesParser, Command};
use regex::Regex;
use walkdir::WalkDir;
use std::error::Error;

type DynErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
enum FileEntityType {
    File,
    Dir,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    types: Vec<FileEntityType>,
}

pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("uniq")
        .version("1.0")
        .author("FallenGameR")
        .about("Finds files and folders in the file system")
        .args([
            arg!([PATH] ... "Paths that would be used to start the search from").default_value("."),
            arg!(-n --name [NAME] ... "File names to look for"),
            arg!(-t --type [TYPE] ... "File types to look for")
                .value_parser(PossibleValuesParser::new(&["f", "d", "l"])),
        ])
        .get_matches();

    let names: Vec<String> = matches.remove_many("name").unwrap_or_default().collect();
    let types: Vec<String> = matches.remove_many("type").unwrap_or_default().collect();

    Ok(Config {
        paths: matches
            .remove_many("PATH")
            .expect("Paths were not provided")
            .collect(),
        names: names
            .into_iter()
            .map(|regex_text| {
                Regex::new(&regex_text)
                    .map_err(|err| format!("Invalid --name '{}', {}", regex_text, err))
            })
            .collect::<Result<_, _>>()?,
        types: types
            .into_iter()
            .map(|entity_type| match entity_type.as_str() {
                "f" => Ok(File),
                "d" => Ok(Dir),
                "l" => Ok(Link),
                unknown => Err(format!("Unsupported file entiry type: {}", unknown)),
                // unreachable! could be used instead here, then we don't need Ok() annotation
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn run(config: Config) -> DynErrorResult<()> {
    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) => println!("{}", entry.path().display()),
                Err(error) => eprint!("Error: {}", error),
            }
        }

    }

    Ok(())
}

/*
fn process_unuque(mut reader: impl BufRead, writer: &mut dyn Write, config: &Config) -> DynErrorResult<()> {
    let mut tracked = String::new();
    let mut current = String::new();
    let mut count = 0;

    let mut output_line = |line: &str, count: usize| -> DynErrorResult<()> {
        if count > 0 {
            let count_str = if config.count {format!("{:>4} ", count)} else {"".to_owned()};
            write!(writer, "{}{}", count_str, line)?;
        }

        Ok(())
    };

    loop {
        // Read line together with line endings
        current.clear();
        let bytes = reader.read_line(&mut current)?;
        if bytes == 0 {
            break;
        }

        if tracked.trim_end() == current.trim_end() {
            // Encountered a duplicate line
            count += 1;
        }
        else {
            // Output previosly tracked line
            output_line(&tracked, count)?;

            // Start tracking the new line
            tracked = current.clone();
            count = 1;
        }
    }

    // The last line was not dumped in the loop
    output_line(&tracked, count)?;

    Ok(())
}

fn open_read(config: &Config) -> DynErrorResult<Box<dyn BufRead>> {
    match config.in_file.as_str() {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        path => Ok(Box::new(BufReader::new(File::open(path)?))),
    }
}

// To check what common traits two types do implement:
// rustup doc "std::io::stdout"
// rustup doc "std::fs::File"
fn open_write(config: &Config) -> DynErrorResult<Box<dyn Write>> {
    match &config.out_file {
        Some(path) if path == "-" => Ok(Box::new(io::stdout())),
        Some(path) => Ok(Box::new(File::create(path)?)),
        None => Ok(Box::new(io::stdout())),
    }
}
*/
