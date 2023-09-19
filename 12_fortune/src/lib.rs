use anyhow::Result;
use clap::{arg, Command};
use regex::Regex;

#[derive(Debug)]
pub struct Config {
    files: Vec<String>,
    regex: Option<Regex>,
    random_seed: Option<u64>,
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
            arg!(-s --seed <RANDOM_SEED> "Random seed to use for the random number generator"),
        ])
        .get_matches();

    // Construct config
    Ok(Config {
        files: matches.remove_many("FILES").expect("At least one file must be provided").collect(),
        lines: matches.remove_one("lines").expect("Default value is provided"),
        bytes: matches.remove_one("bytes"),
        quiet: matches.get_flag("quiet"),
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

/*

// To make it faster we need to read from the end of the file and use IoSlice for output
// Or use File::seek =)
fn print_tail(file: &str, position: &Position, total: Total) -> Result<()> {
    // Variables that are different for printing the tail for line or bytes
    let (size, name, filter): (_,_, &dyn Fn(u8) -> bool) = match total {
        Total::Bytes(bytes) => (bytes, "byte", &|_| true),
        Total::Lines(lines) => (lines, "line", &|b| b == b'\n'),
    };

    // Print error for invalid positions but don't terminate the program
    let Some(offset) = get_offset(position, size) else {
        eprintln!("{position:?}: invalid {name} position for file {file}");
        return Ok(());
    };

    // Rewinding the byte streem to the needed position and take till the end
    let mut skipped = 0;
    let bytes = BufReader::new(File::open(file)?)
    .bytes()
    .filter_map(Result::ok)
    .skip_while(|&b| {
        if skipped == offset {
            return false;
        }

        if filter(b) {
            skipped += 1;
        }

        return true;
    })
    .collect::<Vec<u8>>();

// Output the result to stdout
let mut stdout = std::io::stdout();
stdout.write_all(bytes.as_slice())?;
stdout.flush()?;

Ok(())
}

fn count_bytes(path: &str) -> Result<usize> {
    Ok(std::fs::metadata(path)?.len().try_into()?)
}

pub fn count_lines(path: &str) -> Result<usize> {
    // This version is slower 0.44s only in (debug)
    // release performance is the same 0.15s (release)
    // Buffered read make a huge difference
    let lines = BufReader::new(File::open(path)?)
    .bytes()
    .filter_map(Result::ok)
    .fold(0, |a, c| a + (c == b'\n') as usize);

// This version is faster only in debug 0.37s (debug)
//let mut lines = 0;
//for byte in BufReader::new(File::open(path)?).bytes() {
    //    let Ok(byte) = byte else {continue;};
    //    if byte == b'\n' {
        //        lines += 1;
        //    }
        //}

        Ok(lines)
    }

    // indexes  01234
    // total    5
    // position uses 0..=5 and it offset from end or begining
    // head     0..=4 from what index to start till the end, e.g. 1 results in 1..5
    // tail     1..=5 how many elements to show from the end, e.g. 1 results in 4..5
    // in case when position is counted from tail and the range is going to
    // be more then full file we return range that covers the whole file
    fn get_offset(position: &Position, total: usize) -> Option<usize> {
        let offset = match position {
            Position::FromHead(offset) => *offset,
            Position::FromTail(elements) => total.saturating_sub(*elements),
        };

        if offset >= total { None } else { Some(offset) }
    }
*/
