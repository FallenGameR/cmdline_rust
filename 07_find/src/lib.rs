use crate::FileEntityType::*;
use clap::{arg, builder::PossibleValuesParser, Command};
use regex::Regex;
use walkdir::WalkDir;
use std::error::Error;

type DynErrorResult<T> = Result<T, Box<dyn Error>>;

#[derive(Debug)]
enum FileEntityType {
    File,
    Dir,
    Link,
}

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    names: Vec<Regex>,
    types: Vec<FileEntityType>,
}

pub fn get_args() -> DynErrorResult<Config> {
    let mut matches = Command::new("uniq")
        .version("1.0")
        .author("FallenGameR")
        .about("Finds files and folders in the file system")
        .args([
            arg!([PATH] ... "Paths that would be used to start the search from").default_value("."),
            arg!(-n --name [NAME] ... "File names to look for"),
            arg!(-t --type [TYPE] ... "File types to look for")
                .value_parser(PossibleValuesParser::new(&["f", "d", "l"])),
        ])
        .get_matches();

    let names: Vec<String> = matches.remove_many("name").unwrap_or_default().collect();
    let types: Vec<String> = matches.remove_many("type").unwrap_or_default().collect();

    Ok(Config {
        paths: matches
            .remove_many("PATH")
            .expect("Paths were not provided")
            .collect(),
        names: names
            .into_iter()
            .map(|regex_text| {
                Regex::new(&regex_text)
                    .map_err(|err| format!("Invalid --name '{}', {}", regex_text, err))
            })
            .collect::<Result<_, _>>()?,
        types: types
            .into_iter()
            .map(|entity_type| match entity_type.as_str() {
                "f" => Ok(File),
                "d" => Ok(Dir),
                "l" => Ok(Link),
                unknown => Err(format!("Unsupported file entiry type: {}", unknown)),
                // unreachable! could be used instead here, then we don't need Ok() annotation
            })
            .collect::<Result<_, _>>()?,
    })
}

pub fn run(config: Config) -> DynErrorResult<()> {
    for path in config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) => println!("{}", entry.path().display()),
                Err(error) => eprint!("Error: {}", error),
            }
        }
    }

    Ok(())
}
