use anyhow::{anyhow, bail, Result};
use clap::{arg, Arg, ArgAction, Command};
use std::borrow::Cow;
use std::io::{BufRead, BufReader};
use std::cmp::Ordering::*;

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

    let mut a = file1.next().transpose()?;
    let mut b = file2.next().transpose()?;

    loop {
        // Exit condition
        if a.is_none() && b.is_none() {
            break;
        }

        // Trivial cases
        if a.is_none() {
            output(&config, 2, b.as_deref().expect("Can't be None"));
            b = file2.next().transpose()?;
            continue;
        }

        if b.is_none() {
            output(&config, 1, a.as_deref().expect("Can't be None"));
            a = file1.next().transpose()?;
            continue;
        }

        // Comparison
        let mut a_text = Cow::Borrowed(a.as_deref().expect("Can't be None"));
        let mut b_text = Cow::Borrowed(b.as_deref().expect("Can't be None"));

        if config.case_insensitive {
            a_text = a_text.to_lowercase().into();
            b_text = b_text.to_lowercase().into();
        }

        match a_text.cmp(&b_text) {
            Equal => {
                output(&config, 3, a_text.as_ref());
                a = file1.next().transpose()?;
                b = file2.next().transpose()?;
            },
            Less => {
                output(&config, 1, a_text.as_ref());
                a = file1.next().transpose()?;
            },
            Greater => {
                output(&config, 2, b_text.as_ref());
                b = file2.next().transpose()?;
            },
        }
    }

    Ok(())
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
        2 => config.show_col1 as u8,
        3 => config.show_col1 as u8 + config.show_col2 as u8,
        _ => panic!("Invalid column number"),
    };

    // Print value in the corresponding column
    let delimeters = config.delimeter.repeat(number_of_delimeters as usize);
    println!("{delimeters}{value}");
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