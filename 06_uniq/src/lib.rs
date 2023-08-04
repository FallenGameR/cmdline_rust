use clap::{arg, Command};
use std::{
    error::Error,
    io::{BufRead, BufReader},
};

type DynErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    in_file: String,
    out_file: Option<String>,
    count: bool,
}

pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("uniq")
        .version("1.0")
        .author("FallenGameR")
        .about("Removes adjacent duplicated lines from a file")
        .args([
            arg!([INPUT_FILE] "Input file to process, stdin is -").default_value("-"),
            arg!(-o --output [OUTPUT_FILE] "Output file, stdout if absent"),
            arg!(-c --count "Print duplication count for every line"),
        ])
        .get_matches();

    Ok(Config {
        in_file: matches.remove_one("INPUT_FILE").expect("Input file not provided"),
        out_file: matches.remove_one("output"),
        count: matches.get_flag("count"),
    })
}

// cargo run -- -n (ls .\tests\inputs\*.txt)
// cargo run -- -n (walker .\tests\inputs\ -a)
pub fn run(config: Config) -> DynErrorResult<()> {
    println!("{:?}", config);

    match open(&config.in_file) {
        Err(error) => eprintln!("Can't open file '{}', error {}", &config.in_file, error),
        Ok(_) =>
        {
            println!("Opened file '{}'", &config.in_file);
        },
    }

    Ok(())
}

fn process_stats(mut reader: impl BufRead) -> DynErrorResult<()> {
    let mut line = String::new();

    loop {
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }

        line.clear();
    }

    Ok(())
}

fn open(path: &str) -> DynErrorResult<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(path)?))),
    }
}