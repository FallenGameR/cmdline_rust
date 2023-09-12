use anyhow::{anyhow, bail, Result};
use clap::{arg, Arg, ArgAction, Command};
use std::io::{BufRead, BufReader};

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

pub fn run(config: Config) -> Result<()> {
    // Open iterators files
    let mut file1 = open(&config.file1)?.lines();
    let mut file2 = open(&config.file2)?.lines();

    let report = |column: u8, value: String| match column {
        1 => {
            if config.show_col1 {
                print!("{}\n", value)
            }
        }
        2 => {
            if config.show_col2 {
                print!("{}\n", value)
            }
        }
        3 => {
            if config.show_col3 {
                print!("{}\n", value)
            }
        }
        _ => panic!("Invalid column number"),
    };

    loop {
        // Read lines
        let mut current1 = file1.next().transpose()?;
        let mut current2 = file2.next().transpose()?;

        // Exhaust other file if it's counterpart is exhausted
        let mut current1 = match current1 {
            Some(value) => Some(value),
            None => {
                while let Some(value) = current2 {
                    report(2, value);
                    current2 = file2.next().transpose()?;
                }
                break
            }
        };
        let mut current2 = match current2 {
            Some(value) => Some(value),
            None => {
                while let Some(value) = current1 {
                    report(1, value);
                    current1 = file1.next().transpose()?;
                }
                break
            }
        };
    }

    /*
    // Files to process
    let files = find_files(&config.files, config.recurse);

    // Output should be prepended with file name in case we have many files
    let output = |path: &str, value: &str| {
        if files.len() > 1 {
            print!("{path}:{value}");
        } else {
            print!("{value}");
        }
    };

    // Process each file
    for path in &files {
        // Print per-file error without terminating the program
        let Ok(path) = path else {
            eprintln!("{}", path.as_ref().unwrap_err());
            continue;
        };

        // Open reader to the file
        let reader = match open(path) {
            Ok(reader) => reader,
            Err(error) => {
                eprintln!("Can't open file '{}', error {}", &path, error);
                continue;
            }
        };

        // Process matches
        let lines = find_lines(reader, &config.pattern, config.invert_match)?;
        if config.count {
            output(path, &format!("{}\n", lines.len()));
        } else {
            lines.into_iter().for_each(|line| output(path, &line));
        };
    }

    // Made it to the end without terminating errors
    */
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

/*

fn find_lines(
    mut reader: impl BufRead,
    pattern: &Regex,
    invert_match: bool,
) -> Result<Vec<String>> {
    let mut results = Vec::new();
    let mut line = String::new();

    loop {
        // Read line together with line endings
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        // It should either be a match or it is not a match and we are looking for not-matching lines
        if pattern.is_match(&line) ^ invert_match {
            // Avoiding clone by taking ownership of the line
            // The old line content would be swapped with an empty string here
            results.push(std::mem::take(&mut line));
        } else {
            // If we didn't use the line we need to clean it up for the next iteration
            line.clear();
        }
    }

    Ok(results)
}
*/
