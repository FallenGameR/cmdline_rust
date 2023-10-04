use clap::{arg, Command};
use std::{
    error::Error,
    io::{BufRead, BufReader},
};

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    use_long_format: bool,
    show_hidden: bool,
}

pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("ls")
        .version("1.0")
        .author("FallenGameR")
        .about("List file system entries")
        .args([
            arg!([PATHS] ... "Paths to process, current folder is -").default_value("."),
            arg!(-l --long "Use long format that shows each entry per line"),
            arg!(-h --hidden "Show hidden file system entries"),
        ])
        .get_matches();

    Ok(Config {
        paths: matches
            .remove_many("FILES")
            .expect("No paths provided")
            .collect(),
        use_long_format: matches.get_flag("long"),
        show_hidden: matches.get_flag("hidden"),
    })
}

pub fn run(config: Config) -> Result<()> {
    dbg!(config);
    Ok(())
}