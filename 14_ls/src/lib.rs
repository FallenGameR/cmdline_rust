use anyhow::Result;
use chrono::{DateTime, Utc};
use clap::{arg, Command};
use std::{
    fs,
    os::unix::prelude::MetadataExt,
    path::{Path, PathBuf},
};
use tabular::{Row, Table};

const READ: u32 = 0b100;
const WRITE: u32 = 0b010;
const EXECUTE: u32 = 0b001;
const OTHER: u32 = 0;
const GROUP: u32 = 3;
const USER: u32 = 6;

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
    let paths = find_files(&config.paths, config.show_hidden);

    if config.use_long_format {
        println!("{output}", output = format_output(&paths)?);
    } else {
        for path in paths {
            println!("{:8}", path.display());
        }
    }

    Ok(())
}

fn find_files(paths: &[String], include_hidden: bool) -> Vec<PathBuf> {
    let mut result = Vec::with_capacity(paths.len());

    // Test if we should return that path while honoring the hidden flag
    let should_return = |path: &Path| -> bool {
        if include_hidden {
            return true;
        }

        let Some(file_name) = path.file_name() else {
            return true;
        };

        let is_hidden = file_name.to_string_lossy().starts_with('.');
        !is_hidden
    };

    for path in paths {
        // Making sure the path exists
        let meta = match fs::metadata(path) {
            Ok(meta) => meta,
            Err(error) => {
                eprintln!("{path}: {error}");
                continue;
            }
        };

        // Return explicitly passed and existing file paths right away
        // We would ignore even the hidden flag here
        if meta.is_file() {
            result.push(PathBuf::from(path));
            continue;
        }

        // Make sure folder can be read
        let dir = match fs::read_dir(path) {
            Ok(dir) => dir,
            Err(error) => {
                eprintln!("{path}: {error}");
                continue;
            }
        };

        // Return child paths of the folder
        for entry in dir {
            match entry {
                Err(error) => eprintln!("{error}: Can't enumerate this file system entry"),
                Ok(entry) => {
                    let path = entry.path();
                    if should_return(&path) {
                        result.push(path);
                    }
                }
            }
        }
    }

    result
}

fn format_output(paths: &[PathBuf]) -> Result<String> {
    //         1   2     3     4     5     6     7     8
    let fmt = "{:<}{:<}  {:>}  {:<}  {:<}  {:>}  {:<}  {:<}";
    let mut table = Table::new(fmt);

    for path in paths {
        let meta = path.metadata()?;

        let kind = if meta.is_dir() { "d" } else { "-" };
        let mode = format_mode(meta.mode());
        let links = meta.nlink();
        let user = users::get_user_by_uid(meta.uid())
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or(meta.uid().to_string());
        let group = users::get_group_by_gid(meta.gid())
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or(meta.gid().to_string());
        let length = meta.len();
        let modified = meta.modified()?;
        let modified: DateTime<Utc> = modified.into();
        let modified = modified.format("%Y-%b-%d %H:%M");

        table.add_row(
            Row::new()
                .with_cell(kind) // 1 - directory or else
                .with_cell(mode) // 2 - rwx permissions
                .with_cell(links) // 3 - number of hard links
                .with_cell(user) // 4 - onwer user name
                .with_cell(group) // 5 - owner group name
                .with_cell(length) // 6 - file size in bytes
                .with_cell(modified) // 7 - last modified date
                .with_cell(path.display()), // 8 - path
        );
    }

    Ok(format!("{table}"))
}

fn format_mode(mode: u32) -> String {
    let render = |part: u32| -> String {
        let mut result = String::with_capacity(3);

        let print = |mask: u32, char: char| -> char {
            if part & mask != 0 {
                char
            } else {
                '-'
            }
        };

        result.push(print(READ, 'r'));
        result.push(print(WRITE, 'w'));
        result.push(print(EXECUTE, 'x'));
        result
    };

    let mut result = String::with_capacity(9);
    result.push_str(&render(mode >> USER));
    result.push_str(&render(mode >> GROUP));
    result.push_str(&render(mode >> OTHER));
    result
}

// --------------------------------------------------
#[cfg(test)]
mod test {
    use super::{find_files, format_mode, format_output};
    use std::path::PathBuf;

    #[test]
    fn test_find_files() {
        // Find all non-hidden entries in a directory
        let res = find_files(&["tests/inputs".to_string()], false);
        let mut filenames: Vec<_> = res
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
        let filenames: Vec<_> = res
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
        let mut filenames: Vec<_> = res
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
        let mut filenames: Vec<_> = res
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
        let lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
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
        let mut lines: Vec<&str> = out.split('\n').filter(|s| !s.is_empty()).collect();
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
    fn test_format_mode() {
        assert_eq!(format_mode(0o755), "rwxr-xr-x");
        assert_eq!(format_mode(0o421), "r---w---x");
    }
}
