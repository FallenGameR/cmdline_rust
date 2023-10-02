mod date;
use ansi_term::Style;
use anyhow::{Ok, Result, bail};
use chrono::{Datelike, Local, NaiveDate, Weekday};
use clap::{arg, Command};
use date::{Date, Month, Year};

// Field sizes reflect choises in the chrono crate
#[derive(Debug)]
pub struct Config {
    today: NaiveDate,
    month: Month,
    year: Year,
    show_full_year: bool,
}

pub fn get_args() -> Result<Config> {
    // CLI arguments
    // We try to mimic 'wsl ncal 10 2023 -b'
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
    let explicit_month = date.map(|d| d.month).flatten().or(month);

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
        show_full_year: matches.get_flag("show_full_year") || (explicit_month.is_none() && explicit_year.is_some()),
    })
}

pub fn run(config: Config) -> Result<()> {
    match config.show_full_year {
        false => {
            // When only a single month is shown we add the year into the header
            for line in format_month(config.year.0, config.month.0, true, config.today) {
                println!("{line}");
            }
        }
        true => {
            // When we render the whole year we use a special header
            println!("{year:^44}", year = config.year.0);
        },
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
fn format_month(year: i32, month: u32, add_year_annitation: bool, highlighted_day: NaiveDate) -> Vec<String> {
    let mut result = Vec::new();
    let mut date = NaiveDate::from_ymd_opt(year, month, 1).expect("Date must be valid");

    // Header
    let month_text = date::MONTH_NAMES
        .get(date.month0() as usize)
        .expect("Date must be valid");
    let header_text = if add_year_annitation {
        format!("{month_text} {year}")
    } else {
        format!("{month_text}")
    };
    let header_text = format!("{header_text:^20}  ");
    result.push(header_text);

    // Week processor - iterates over week days startung from week_start
    let week_start = Weekday::Sun;
    let mut weekday = week_start;
    let mut process_week = |process_day: &mut dyn FnMut(Weekday) -> String| -> String {
        let mut line = String::new();
        loop {
            line.push_str(&process_day(weekday));
            weekday = weekday.succ();
            if weekday == week_start {
                break;
            }
        }
        line.push_str(" ");
        line
    };

    // Labels
    result.push(process_week(&mut |weekday: Weekday| -> String {
        let mut text = weekday.to_string();
        text.truncate(2);
        format!("{:2} ", text)
    }));

    // Dates table
    let processed_month = date.month0();
    while date.month0() == processed_month {
        result.push(process_week(&mut |weekday: Weekday| -> String {
            // Space padding
            if date.weekday() != weekday || date.month0() != processed_month {
                return format!("{:2} ", " ");
            }

            // Current day highlight
            let mut text = format!("{:2}", date.day0() + 1);
            if date == highlighted_day {
                text = Style::default().reverse().paint(text).to_string();
            }
            text.push(' ');

            // Rewind to next date, return currently rendered one
            date = date.succ_opt().expect("Date must be valid");
            text
        }));
    }

    // Pad for 8 lines per week representation
    for _ in result.len()..8 {
        result.push(format!("{:20}  ", " "));
    }

    result
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::format_month;
    use chrono::NaiveDate;

    #[test]
    fn test_format_month() {
        let today = NaiveDate::from_ymd_opt(0, 1, 1).unwrap();
        let leap_february = vec![
            "   February 2020      ",
            "Su Mo Tu We Th Fr Sa  ",
            "                   1  ",
            " 2  3  4  5  6  7  8  ",
            " 9 10 11 12 13 14 15  ",
            "16 17 18 19 20 21 22  ",
            "23 24 25 26 27 28 29  ",
            "                      ",
        ];
        assert_eq!(format_month(2020, 2, true, today), leap_february);

        let may = vec![
            "        May           ",
            "Su Mo Tu We Th Fr Sa  ",
            "                1  2  ",
            " 3  4  5  6  7  8  9  ",
            "10 11 12 13 14 15 16  ",
            "17 18 19 20 21 22 23  ",
            "24 25 26 27 28 29 30  ",
            "31                    ",
        ];
        assert_eq!(format_month(2020, 5, false, today), may);

        let april_hl = vec![
            "     April 2021       ",
            "Su Mo Tu We Th Fr Sa  ",
            "             1  2  3  ",
            " 4  5  6 \u{1b}[7m 7\u{1b}[0m  8  9 10  ",
            "11 12 13 14 15 16 17  ",
            "18 19 20 21 22 23 24  ",
            "25 26 27 28 29 30     ",
            "                      ",
        ];
        let today = NaiveDate::from_ymd_opt(2021, 4, 7).unwrap();
        assert_eq!(format_month(2021, 4, true, today), april_hl);
    }
}
