use std::error::Error;

use clap::{arg, Command};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn run(config: Config) -> MyResult<()> {
    dbg!(&config);
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let mut matches = Command::new("cat")
        .version("1.0")
        .author("FallenGameR")
        .about("Concatenates files and optionally add line numbers")
        .args([
            arg!([files] ... "Input files to concatenate, stdin if empty").default_value("-"),
            arg!(-n --number_lines "Add line numbers to output")
                .conflicts_with("number_nonblank_lines"),
            arg!(-b --number_nonblank_lines "Number only nonblack lines")
                .conflicts_with("number_lines"),
        ])
        .get_matches();

    Ok(Config {
        files: matches
            .remove_many("files")
            .expect("No file paths provided")
            .collect(),
        number_lines: matches.get_flag("number_lines"),
        number_nonblank_lines: matches.get_flag("number_nonblank_lines"),
    })
}
