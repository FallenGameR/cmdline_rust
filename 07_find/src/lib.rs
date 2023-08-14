use clap::{arg, builder::PossibleValuesParser, Command};
use crate::FileEntityType::*;
use regex::Regex;
use std::error::Error;
use walkdir::WalkDir;

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
    for path in &config.paths {
        for entry in WalkDir::new(path) {
            match entry {
                Ok(entry) => process(entry, &config),
                Err(error) => eprint!("Error: {}", error),
            }
        }
    }

    Ok(())
}

fn process(entry: walkdir::DirEntry, config: &Config) {
    let path = entry.path();
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let name_match =
        config.names.is_empty() ||
        config.names.iter().any(|regex| regex.is_match(file_name));
    let type_match =
        config.types.is_empty() ||
        config.types.iter().any(|entity_type| match entity_type {
            File => entry.file_type().is_file(),
            Dir => entry.file_type().is_dir(),
            Link => entry.file_type().is_symlink(),
        });

    if name_match && type_match {
        println!("{}", path.display());
    }
}
