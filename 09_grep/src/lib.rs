use anyhow::{bail, Result};
use clap::{arg, Command};
use regex::RegexBuilder;
use std::io::{BufRead, BufReader};

#[derive(Debug)]
pub struct Config {
    pattern: String,
    files: Vec<String>,
    recurse: bool,
    insensitive: bool,
    count: bool,
    invert_match: bool,
}

pub fn run(config: Config) -> Result<()> {
    dbg!(&config);

    let pattern = RegexBuilder::new(&config.pattern)
        .case_insensitive(config.insensitive)
        .build()?;

    dbg!(pattern.as_str());

    for path in &config.files {
        match open(path) {
            Err(error) => eprintln!("Can't open file '{}', error {}", &path, error),
            Ok(reader) => process_file(path, reader, &config)?,
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
    let mut matches = Command::new("grep")
        .version("1.0")
        .author("FallenGameR")
        .about("Finds text specified by regular expression in files")
        .args([
            arg!(<REGULAR_EXPRESSION> "Regular expression to use"),
            arg!([FILES] ... "Files or folders to process, stdin is -").default_value("-"),
            arg!(-r --recurse "Recuresivelly descend into folders looking for files"),
            arg!(-i --insensitive "Use case insensitive regex matching"),
            arg!(-c --count "Just count the matches, don't show them"),
            arg!(-v --invert_match "Find lines that don't match the regular expression"),
        ])
        .get_matches();

    // Construct config
    Ok(Config {
        pattern: matches
            .remove_one("REGULAR_EXPRESSION")
            .expect("No pattern provided"),
        files: matches
            .remove_many("FILES")
            .expect("No file paths provided")
            .collect(),
        recurse: matches.get_flag("recurse"),
        insensitive: matches.get_flag("insensitive"),
        count: matches.get_flag("count"),
        invert_match: matches.get_flag("invert_match"),
    })
}

fn process_file(path: &str, reader: Box<dyn BufRead>, _: &Config) -> Result<()> {
    for line in reader.lines() {
        match line {
            Err(error) => eprintln!("Can't read line from file '{path}', error {error}"),
            Ok(_) => {
                todo!("Process the line")
            }
        }
    }

    Ok(())
}
