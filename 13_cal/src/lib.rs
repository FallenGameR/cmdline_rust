mod date;
use ansi_term::{Style, Color};
use anyhow::{bail, Ok, Result};
use chrono::{Datelike, Local, NaiveDate, Weekday};
use clap::{arg, Command};
use date::{Date, Month, Year};

const DAY_WIDTH: usize = 2;
const WEEK_WIDTH: usize = 20;
const YEAR_WIDTH: usize = 70;
const WEEK_HEIGHT: usize = 8;
const MONTHS_IN_YEAR: u32 = 12;
const YEAR_WIDTH_IN_COLUMNS: usize = 3;
const WEEK_START: Weekday = Weekday::Mon;

#[derive(Debug)]
pub struct Config {
    today: NaiveDate,
    month: Month,
    year: Year,
    show_full_year: bool,
}

pub fn get_args() -> Result<Config> {
    // CLI arguments
    // As a start I tried to mimic 'wsl ncal 10 2023 -b'
    let mut matches = Command::new("cal")
        .version("1.0")
        .author("FallenGameR")
        .about("Proleptic Gregorian month calendar with coloring")
        .args([
            arg!([DATE]... "Year number (1-9999) or month name or number (1-12) followed by year number")
                .help_heading("DATE as [[month] year]"),
            arg!(-m --month <MONTH> "Month name or number (1-12)\nIs ignored if DATE specifies month")
                .value_parser(Date::parse_month),
            arg!(-y --show_full_year "Show calendar for the whole year")
                .conflicts_with("month"),
        ])
        .get_matches();

    // Parse arguments
    let today = Local::now().naive_local();
    let month = matches.remove_one("month");
    let date_parts: Vec<String> = matches.remove_many("DATE").unwrap_or_default().collect();
    let date = match date_parts.as_slice() {
        [] => None,
        parts => Some(Date::parse(&parts.join(" "))?),
    };
    let explicit_year = date.map(|d| d.year);
    let explicit_month = date.and_then(|d| d.month).or(month);

    // Sanity check, can't specify month twice
    if date.is_some_and(|d| d.month.is_some()) && month.is_some() {
        bail!("Can't specify month twice");
    }

    // Construct config
    //
    // Month is resolved in steps:
    // - from DATE if specified
    // - from MONTH if sepecified
    // - otherwise it is current month
    //
    // Year is resolved in steps:
    // - from DATE if specified
    // - otherwise it is current year
    //
    // Show full year is resolved in steps:
    // - from CLI flag
    // - if month was not specified neither via DATE nor via MONTH _and_
    //   year was explicitly specified (so when no arguments are given we show only the current month)
    // - otherwise it is false
    //
    Ok(Config {
        today: today.date(),
        month: explicit_month.unwrap_or(Month(today.month())),
        year: explicit_year.unwrap_or(Year(today.year())),
        show_full_year: matches.get_flag("show_full_year")
            || (explicit_month.is_none() && explicit_year.is_some()),
    })
}

pub fn run(config: Config) -> Result<()> {
    // Rendering a single month annotated with year
    if !config.show_full_year {
        for line in format_month(config.year.0, config.month.0, Some(config.today), true, true) {
            println!("{line}");
        }
        return Ok(());
    }

    // Rendering the whole year, 3 months per line
    println!("{year:^YEAR_WIDTH$}", year = Style::default().underline().paint(config.year.0.to_string()).to_string());
    println!();

    let months = (1..=MONTHS_IN_YEAR)
        .map(|month| format_month(config.year.0, month, Some(config.today), false, true))
        .collect::<Vec<_>>();
    let months_chunks = months.chunks(YEAR_WIDTH_IN_COLUMNS);
    let last_chunk_index = months_chunks.len() - 1;

    for (index, chunk) in months_chunks.enumerate() {
        let [first, second, third] = chunk else {
            bail!("Invalid chunk")
        };

        for line in 0..WEEK_HEIGHT {
            print!("{}", first[line]);
            print!("{}", second[line]);
            print!("{}", third[line]);
            println!();
        }

        if index < last_chunk_index {
            println!();
        }
    }

    Ok(())
}

/// Renders a month as a vector of strings, 7 rows, 22 chars each
///
/// - `year` and `month` - identify the month to render
/// - `add_year_annitation` - if true, year is added to the header
/// - `highlighted_day` - date that would be highlighted (usually today)
///
/// # Example
///
/// ```ignore
/// [0] "   February 2020      "
/// [1] "Su Mo Tu We Th Fr Sa  "
/// [2] "                   1  "
/// [3] " 2  3  4  5  6  7  8  "
/// [4] " 9 10 11 12 13 14 15  "
/// [5] "16 17 18 19 20 21 22  "
/// [6] "23 24 25 26 27 28 29  "
/// [7] "                      "
/// ```
fn format_month(
    year: i32,
    month: u32,
    highlighted_day: Option<NaiveDate>,
    do_year_annotation: bool,
    do_colorization: bool,
) -> Vec<String> {
    let mut result = Vec::with_capacity(WEEK_HEIGHT);
    let mut date = NaiveDate::from_ymd_opt(year, month, 1).expect("Date must be valid");

    // Header
    let month_text = date::MONTH_NAMES
        .get(date.month0() as usize)
        .expect("Date must be valid");
    let mut header_text = if do_year_annotation {
        format!("{month_text} {year}")
    } else {
        format!("{month_text}")
    };
    header_text = format!("{header_text:^WEEK_WIDTH$}  ");
    if do_colorization {
        header_text = Color::Cyan.bold().paint(header_text).to_string();
    }
    result.push(header_text);

    // Week processor - iterates over week days starting from week_start
    let mut weekday = WEEK_START;
    let mut process_week = |process_day: &mut dyn FnMut(Weekday) -> String| -> String {
        let mut line = String::new();
        loop {
            line.push_str(&process_day(weekday));
            weekday = weekday.succ();
            if weekday == WEEK_START {
                break;
            }
        }
        line.push(' ');
        line
    };

    // Labels
    result.push(process_week(&mut |weekday: Weekday| -> String {
        let mut day_name = weekday.to_string();
        day_name.truncate(DAY_WIDTH);
        if do_colorization {
            day_name = Color::Cyan.normal().paint(day_name).to_string();
        }
        format!("{day_name:DAY_WIDTH$} ")
    }));

    // Dates table
    let processed_month = date.month0();
    while date.month0() == processed_month {
        result.push(process_week(&mut |weekday: Weekday| -> String {
            // Space padding
            if date.weekday() != weekday || date.month0() != processed_month {
                return format!("{:DAY_WIDTH$} ", " ");
            }

            // Current day highlight
            let mut text = format!("{:DAY_WIDTH$}", date.day0() + 1);
            if Some(date) == highlighted_day {
                text = Style::default().reverse().paint(text).to_string();
            }
            text.push(' ');

            // Rewind to next date, return currently rendered one
            date = date.succ_opt().expect("Date must be valid");
            text
        }));
    }

    // Insert spaces for uniform week text representation
    result.resize_with(WEEK_HEIGHT, || format!("{:WEEK_WIDTH$}  ", " "));
    result
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::format_month;
    use chrono::NaiveDate;

    #[test]
    fn test_format_month() {
        let leap_february = vec![
            "   February 2020      ",
            "Mo Tu We Th Fr Sa Su  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29     ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, None, true, false), leap_february);

        let may = vec![
            "        May           ",
            "Mo Tu We Th Fr Sa Su  ",
            "             1  2  3  ",
            " 4  5  6  7  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30 31  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 5, None, false, false), may);

        let april_hl = vec![
            "     April 2021       ",
            "Mo Tu We Th Fr Sa Su  ",
            "          1  2  3  4  ",
            " 5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10 11  ",
            "12 13 14 15 16 17 18  ",
            "19 20 21 22 23 24 25  ",
            "26 27 28 29 30        ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, Some(today), true, false), april_hl);
    }
}
