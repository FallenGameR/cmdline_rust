use clap::{arg, Command};
use std::{
    error::Error,
    io::{BufRead, BufReader, Write, self}, fs::File,
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

    let mut writer = open_write(&config)?;

    match open_read(&config) {
        Err(error) => panic!("Can't open file '{}', error {}", &config.in_file, error),
        Ok(reader) => process_unuque(reader, &mut writer, &config)?,
    }

    Ok(())
}

fn process_unuque(mut reader: impl BufRead, writer: &mut dyn Write, config: &Config) -> DynErrorResult<()> {
    let mut tracked = String::new();
    let mut current = String::new();
    let mut count = 0;

    let mut output_line = |line: &str, count: usize| {
        if count > 0 {
            let count_str = if config.count {format!("{:>4} ", count)} else {"".to_owned()};
            write!(writer, "{}{}", count_str, line).expect("It should be possible to write to stdout or file");
        }
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
            output_line(&tracked, count);

            // Start tracking the new line
            tracked = current.clone();
            count = 1;
        }
    }

    // The last line was not dumped in the loop
    output_line(&tracked, count);

    Ok(())
}

fn open_read(config: &Config) -> DynErrorResult<Box<dyn BufRead>> {
    match config.in_file.as_str() {
        "-" => Ok(Box::new(BufReader::new(io::stdin()))),
        path => Ok(Box::new(BufReader::new(File::open(path)?))),
    }
}

fn open_write(config: &Config) -> DynErrorResult<Box<dyn Write>> {
    match &config.out_file {
        Some(path) if path == "-" => Ok(Box::new(io::stdout())),
        Some(path) => Ok(Box::new(File::create(path)?)),
        None => Ok(Box::new(io::stdout())),
    }
}
