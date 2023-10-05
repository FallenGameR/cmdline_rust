mod owner;

use clap::{arg, Command};
use anyhow::Result;
use owner::Owner;
use std::{path::PathBuf, fs};

#[derive(Debug)]
pub struct Config {
    paths: Vec<String>,
    use_long_format: bool,
    show_hidden: bool,
}

pub fn get_args() -> Result<Config> {
    let mut matches = Command::new("ls")
        .version("1.0")
        .author("FallenGameR")
        .about("List file system entries on Linux")
        .args([
            arg!([PATHS] ... "Paths to process, current folder is .").default_value("."),
            arg!(-l --long "Use long format that shows each entry per line"),
            arg!(-a --all "Show all file system entries, including hidden ones"),
        ])
        .get_matches();

    Ok(Config {
        paths: matches
            .remove_many("PATHS")
            .expect("No paths provided")
            .collect(),
        use_long_format: matches.get_flag("long"),
        show_hidden: matches.get_flag("all"),
    })
}

pub fn run(config: Config) -> Result<()> {
    dbg!(config);
    Ok(())
}

// it would not ever return an error since it handles all the errors by dumping them to STDERR
// the signature needs to be updated
fn find_files(paths: &[String], _show_hidden: bool) -> Result<Vec<PathBuf>> {
    let mut result: Vec<PathBuf> = Vec::with_capacity(paths.len());

    for path in paths {
        let Ok(meta) = fs::metadata(path) else {
            eprintln!("{path}: No such file or directory");
            continue;
        };

        if meta.is_file() {
            result.push(PathBuf::from(path));
            continue;
        }

        let Ok(dir) = fs::read_dir(path) else {
            eprintln!("{path}: Can't read this directory");
            continue;
        };

        for entry in dir {
            match entry {
                Ok(entry) => result.push(entry.path()),
                Err(error) => eprintln!("{error}: Can't enumerate this file system entry"),
            }
        }
    }

    Ok(result)
}

fn format_mode(_mode: u32) -> String {
    todo!()
}

fn format_output(_paths: &[PathBuf]) -> Result<String> {
    todo!()
}

fn mk_triple(_mode: u32, _owner: Owner) -> String {
    todo!()
}

// --------------------------------------------------
#[cfg(test)]
mod test {
    use super::{find_files, format_mode, format_output, mk_triple, Owner};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Find all non-hidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );

        // Any existing file should be found even if hidden
        let res = find_files(&["tests/inputs/.hidden".to_string()], false);
        assert!(res.is_ok());
        let filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        assert_eq!(filenames, ["tests/inputs/.hidden"]);

        // Test multiple path arguments
        let res = find_files(
            &[
                "tests/inputs/bustle.txt".to_string(),
                "tests/inputs/dir".to_string(),
            ],
            false,
        );
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            ["tests/inputs/bustle.txt", "tests/inputs/dir/spiders.txt"]
        );
    }

    #[test]
    fn test_find_files_hidden() {
        // Find all entries in a directory including hidden
        let res = find_files(&["tests/inputs".to_string()], true);
        assert!(res.is_ok());
        let mut filenames: Vec<_> = res
            .unwrap()
            .iter()
            .map(|entry| entry.display().to_string())
            .collect();
        filenames.sort();
        assert_eq!(
            filenames,
            [
                "tests/inputs/.hidden",
                "tests/inputs/bustle.txt",
                "tests/inputs/dir",
                "tests/inputs/empty.txt",
                "tests/inputs/fox.txt",
            ]
        );
    }

    fn long_match(
        line: &str,
        expected_name: &str,
        expected_perms: &str,
        expected_size: Option<&str>,
    ) {
        let parts: Vec<_> = line.split_whitespace().collect();
        assert!(!parts.is_empty() && parts.len() <= 10);

        let perms = parts.first().unwrap();
        assert_eq!(perms, &expected_perms);

        if let Some(size) = expected_size {
            let file_size = parts.get(4).unwrap();
            assert_eq!(file_size, &size);
        }

        let display_name = parts.last().unwrap();
        assert_eq!(display_name, &expected_name);
    }

    #[test]
    fn test_format_output_one() {
        let bustle_path = "tests/inputs/bustle.txt";
        let bustle = PathBuf::from(bustle_path);

        let res = format_output(&[bustle]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let lines: Vec<&str> =
            out.split('\n').filter(|s| !s.is_empty()).collect();
        assert_eq!(lines.len(), 1);

        let line1 = lines.first().unwrap();
        long_match(line1, bustle_path, "-rw-r--r--", Some("193"));
    }

    #[test]
    fn test_format_output_two() {
        let res = format_output(&[
            PathBuf::from("tests/inputs/dir"),
            PathBuf::from("tests/inputs/empty.txt"),
        ]);
        assert!(res.is_ok());

        let out = res.unwrap();
        let mut lines: Vec<&str> =
            out.split('\n').filter(|s| !s.is_empty()).collect();
        lines.sort_unstable();
        assert_eq!(lines.len(), 2);

        let empty_line = lines.remove(0);
        long_match(
            empty_line,
            "tests/inputs/empty.txt",
            "-rw-r--r--",
            Some("0"),
        );

        let dir_line = lines.remove(0);
        long_match(dir_line, "tests/inputs/dir", "drwxr-xr-x", None);
    }

    #[test]
    fn test_mk_triple() {
        assert_eq!(mk_triple(0o751, Owner::User), "rwx");
        assert_eq!(mk_triple(0o751, Owner::Group), "r-x");
        assert_eq!(mk_triple(0o751, Owner::Other), "--x");
        assert_eq!(mk_triple(0o600, Owner::Other), "---");
    }

    #[test]
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }
}
