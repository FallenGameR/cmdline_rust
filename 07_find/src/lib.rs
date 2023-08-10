use clap::{arg, Command, builder::PossibleValuesParser};
use std::error::Error;

type DynErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<String>,
    types: Vec<String>,
}

pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("uniq")
        .version("1.0")
        .author("FallenGameR")
        .about("Finds files and folders in the file system")
        .args([
            arg!(<PATH> ... "Paths that would be used to start the search from"),
            arg!(-n --name [NAME] ... "File names to look for"),
            arg!(-t --type [TYPE] ... "File types to look for")
                .value_parser(PossibleValuesParser::new(&["f", "d", "l"]))
        ])
        .get_matches();

    Ok(Config {
        paths: matches.remove_many("PATH").expect("Paths were not provided").collect(),
        names: matches.remove_many("name").expect("Names were not provided").collect(),
        types: matches.remove_many("type").expect("Types were not provided").collect(),
    })
}

// cargo run -- -n (ls .\tests\inputs\*.txt)
// cargo run -- -n (walker .\tests\inputs\ -a)
pub fn run(config: Config) -> DynErrorResult<()> {

    println!("{:?}", config);

    /*
    match open_read(&config) {
        Err(error) => panic!("Can't open file '{}', error {}", &config.in_file, error),
        Ok(reader) => process_unuque(reader, &mut writer, &config)?,
    }
    */

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