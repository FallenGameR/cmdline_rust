use std::path::PathBuf;

use anyhow::Result;
use clap::{arg, Command};
use regex::{Regex, RegexBuilder};

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    regex: Option<Regex>,
    random_seed: Option<u64>,
}

#[derive(Debug)]
pub struct Fortune {
    file: String,
    text: String,
}

pub fn get_args() -> Result<Config> {
    // CLI arguments
    let mut matches = Command::new("fortune")
        .version("1.0")
        .author("FallenGameR")
        .about("Randomly select a text piece from a set of files")
        .args([
            arg!(<FILES> ... "Files to process"),
            arg!(-m --match <REGULAR_EXPRESSION> "Fortunes would be matched by this regular expression"),
            arg!(-i --insensitive "Use case insensitive regex matching"),
            arg!(-s --seed <RANDOM_SEED> "Random seed to use for the random number generator")
                .value_parser(clap::value_parser!(u64)),
        ])
        .get_matches();

    // Construct regex
    let pattern = matches.remove_one::<String>("match").map(|text| {
        RegexBuilder::new(&text)
            .case_insensitive(matches.get_flag("insensitive"))
            .build()
    });

    // Construct config
    Ok(Config {
        files: matches
            .remove_many("FILES")
            .expect("At least one file must be provided")
            .collect(),
        regex: pattern.transpose()?,
        random_seed: matches.remove_one("seed"),
    })
}

pub fn run(config: Config) -> Result<()> {
    dbg!(&config);

    /*
    let is_header_needed = config.files.len() > 1 && !config.quiet;

    for (index, file) in config.files.iter().enumerate() {
        if is_header_needed {
            let spacer = if index > 0 { "\n" } else { "" };
            println!("{spacer}==> {file} <==");
        }

        match config.bytes.as_ref() {
            Some(bytes) => print_tail(file, &bytes, Total::Bytes(count_bytes(&file)?))?,
            None => print_tail(file, &config.lines, Total::Lines(count_lines(&file)?))?,
        }
    }
    */

    Ok(())
}

fn find_files(_paths: &[String]) -> Result<Vec<PathBuf>> {
    todo!()
}

fn pick_fortune(_fortunes: &[Fortune], _seed: Option<u64>) -> Option<String> {
    todo!()
}

fn read_fortunes(_paths: &[PathBuf]) -> Result<Vec<Fortune>> {
    todo!()
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{
        find_files, pick_fortune, read_fortunes, Fortune,
    };
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Verify that the function finds a file known to exist
        let res = find_files(&["./tests/inputs/jokes".to_string()]);
        assert!(res.is_ok());

        let files = res.unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(
            files.get(0).unwrap().to_string_lossy(),
            "./tests/inputs/jokes"
        );

        // Fails to find a bad file
        let res = find_files(&["/path/does/not/exist".to_string()]);
        assert!(res.is_err());

        // Finds all the input files, excludes ".dat"
        let res = find_files(&["./tests/inputs".to_string()]);
        assert!(res.is_ok());

        // Check number and order of files
        let files = res.unwrap();
        assert_eq!(files.len(), 5);
        let first = files.get(0).unwrap().display().to_string();
        assert!(first.contains("ascii-art"));
        let last = files.last().unwrap().display().to_string();
        assert!(last.contains("quotes"));

        // Test for multiple sources, path must be unique and sorted
        let res = find_files(&[
            "./tests/inputs/jokes".to_string(),
            "./tests/inputs/ascii-art".to_string(),
            "./tests/inputs/jokes".to_string(),
        ]);
        assert!(res.is_ok());
        let files = res.unwrap();
        assert_eq!(files.len(), 2);
        if let Some(filename) = files.first().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "ascii-art".to_string())
        }
        if let Some(filename) = files.last().unwrap().file_name() {
            assert_eq!(filename.to_string_lossy(), "jokes".to_string())
        }
    }

    #[test]
    fn test_read_fortunes() {
        // Parses all the fortunes without a filter
        let res = read_fortunes(&[PathBuf::from("./tests/inputs/jokes")]);
        assert!(res.is_ok());

        if let Ok(fortunes) = res {
            // Correct number and sorting
            assert_eq!(fortunes.len(), 6);
            assert_eq!(
                fortunes.first().unwrap().text,
                "Q. What do you call a head of lettuce in a shirt and tie?\n\
                A. Collared greens."
            );
            assert_eq!(
                fortunes.last().unwrap().text,
                "Q: What do you call a deer wearing an eye patch?\n\
                A: A bad idea (bad-eye deer)."
            );
        }

        // Filters for matching text
        let res = read_fortunes(&[
            PathBuf::from("./tests/inputs/jokes"),
            PathBuf::from("./tests/inputs/quotes"),
        ]);
        assert!(res.is_ok());
        assert_eq!(res.unwrap().len(), 11);
    }

    #[test]
    fn test_pick_fortune() {
        // Create a slice of fortunes
        let fortunes = &[
            Fortune {
                file: "fortunes".to_string(),
                text: "You cannot achieve the impossible without \
                      attempting the absurd."
                    .to_string(),
            },
            Fortune {
                file: "fortunes".to_string(),
                text: "Assumption is the mother of all screw-ups."
                    .to_string(),
            },
            Fortune {
                file: "fortunes".to_string(),
                text: "Neckties strangle clear thinking.".to_string(),
            },
        ];

        // Pick a fortune with a seed
        assert_eq!(
            pick_fortune(fortunes, Some(1)).unwrap(),
            "Neckties strangle clear thinking.".to_string()
        );
    }
}
