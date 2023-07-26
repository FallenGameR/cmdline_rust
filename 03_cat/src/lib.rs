use std::error::Error;

use clap::{Command, arg};

type MyResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    number_lines: bool,
    number_nonblank_lines: bool,
}

pub fn run () -> MyResult<()> {
    println!("Hello, world!");
    Ok(())
}

pub fn get_args() -> MyResult<Config> {
    let mut matches = Command::new("cat")
        .version("1.0")
        .author("FallenGameR")
        .about("Concatenates files and can add line numbers")
        .args([
            arg!(<files> ... "Input files to concatenate"),
            arg!(-n --number_lines "Add line numbers to output"),
            arg!(-b --number_nonblank_lines "Add line numbers only to nonblack lines"),
        ])
        .get_matches();

    Ok(Config {
        files: matches.remove_many("files").expect("No file paths provided").collect(),
        number_lines: matches.get_flag("number_lines"),
        number_nonblank_lines: matches.get_flag("number_nonblank_lines"),
     })
}