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
        Ok(reader) =>
        {
            process_unuque(reader, &config)?;
        },
    }

    //File::create

    Ok(())
}

fn process_unuque(mut reader: impl BufRead, config: &Config) -> DynErrorResult<()> {
    let mut tracked = String::new();
    let mut current = String::new();
    let mut count = 0;

    fn output_line(line: &str, count: usize, config: &Config) {
        let count_str = if config.count {count.to_string() + " "} else {"".to_owned()};
        if count > 0 {
            print!("{}{}", count_str, line);
        }
    }

    loop {
        // Read line together with line endings
        current.clear();
        let bytes = reader.read_line(&mut current)?;
        if bytes == 0 {
            break;
        }

        if tracked == current {
            // Encountered a duplicate line
            count += 1;
        }
        else {
            // Output previosly tracked line
            output_line(&tracked, count, config);

            // Start tracking the new line
            tracked = current.clone();
            count = 1;
        }
    }

    // The last line was not dumped in the loop
    output_line(&tracked, count, config);

    Ok(())
}

fn open(path: &str) -> DynErrorResult<Box<dyn BufRead>> {
    match path {
        "-" => Ok(Box::new(BufReader::new(std::io::stdin()))),
        _ => Ok(Box::new(BufReader::new(std::fs::File::open(path)?))),
    }
}