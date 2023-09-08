use anyhow::{anyhow, bail, Error, Result};
use clap::{arg, Command};
use predicates::path;
use regex::{Regex, RegexBuilder};
use std::{
    fs::DirEntry,
    io::{BufRead, BufReader},
};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct Config {
    pattern: Regex,
    files: Vec<String>,
    recurse: bool,
    count: bool,
    invert_match: bool,
}

pub fn run(config: Config) -> Result<()> {
    dbg!(&config);

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
            arg!(-i --insensitive "Use case insensitive regex matching"),
            arg!(-r --recurse "Recuresivelly descend into folders looking for files"),
            arg!(-c --count "Just count the matches, don't show them"),
            arg!(-v --invert_match "Find lines that don't match the regular expression"),
        ])
        .get_matches();

    // Construct regex
    let pattern_text: String = matches
        .remove_one("REGULAR_EXPRESSION")
        .expect("No pattern provided");
    let pattern = RegexBuilder::new(&pattern_text)
        .case_insensitive(matches.get_flag("insensitive"))
        .build()?;

    // Construct config
    Ok(Config {
        pattern,
        files: matches
            .remove_many("FILES")
            .expect("No file paths provided")
            .collect(),
        recurse: matches.get_flag("recurse"),
        count: matches.get_flag("count"),
        invert_match: matches.get_flag("invert_match"),
    })
}

fn find_files(paths: &[String], recurse: bool) -> Vec<Result<String>> {
    let mut files = Vec::new();

    for path in paths {
        for root in WalkDir::new(path) {
            match root {
                Err(error) => {
                    files.push(Err(error.into()));
                }
                Ok(entry) if entry.file_type().is_file() => {
                    files.push(Ok(entry.path().to_string_lossy().into()));
                }
                Ok(entry) if entry.file_type().is_dir() && !recurse => {
                    let path = entry.path().display();
                    files.push(Err(anyhow!("{path} is a directory")));
                    break;
                }
                Ok(_) => (),
            }
        }
    }

    files
}

fn find_lines(reader: impl BufRead, pattern: &Regex, invert_match: bool) -> Result<Vec<String>> {
    let mut results = Vec::new();

    for line in reader.lines() {
        let line = line?;

        if pattern.is_match(&line) ^ invert_match {
            results.push(line);
        }
    }

    Ok(results)
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

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{find_files, find_lines};
    use rand::{distributions::Alphanumeric, Rng};
    use regex::{Regex, RegexBuilder};
    use std::io::Cursor;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let files = find_files(&["./tests/inputs/fox.txt".to_string()], false);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].as_ref().unwrap(), "./tests/inputs/fox.txt");

        // The function should reject a directory without the recursive option
        let files = find_files(&["./tests/inputs".to_string()], false);
        assert_eq!(files.len(), 1);
        if let Err(e) = &files[0] {
            assert_eq!(e.to_string(), "./tests/inputs is a directory");
        }

        // Verify the function recurses to find four files in the directory
        let res = find_files(&["./tests/inputs".to_string()], true);
        let mut files: Vec<String> = res
            .iter()
            .map(|r| r.as_ref().unwrap().replace("\\", "/"))
            .collect();
        files.sort();
        assert_eq!(files.len(), 4);
        assert_eq!(
            files,
            vec![
                "./tests/inputs/bustle.txt",
                "./tests/inputs/empty.txt",
                "./tests/inputs/fox.txt",
                "./tests/inputs/nobody.txt",
            ]
        );

        // Generate a random string to represent a nonexistent file
        let bad: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        // Verify that the function returns the bad file as an error
        let files = find_files(&[bad], false);
        assert_eq!(files.len(), 1);
        assert!(files[0].is_err());
    }

    #[test]
    fn test_find_lines() {
        let text = b"Lorem\nIpsum\r\nDOLOR";

        // The pattern _or_ should match the one line, "Lorem"
        let re1 = Regex::new("or").unwrap();
        let matches = find_lines(Cursor::new(&text), &re1, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);

        // When inverted, the function should match the other two lines
        let matches = find_lines(Cursor::new(&text), &re1, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // This regex will be case-insensitive
        let re2 = RegexBuilder::new("or")
            .case_insensitive(true)
            .build()
            .unwrap();

        // The two lines "Lorem" and "DOLOR" should match
        let matches = find_lines(Cursor::new(&text), &re2, false);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 2);

        // When inverted, the one remaining line should match
        let matches = find_lines(Cursor::new(&text), &re2, true);
        assert!(matches.is_ok());
        assert_eq!(matches.unwrap().len(), 1);
    }
}
