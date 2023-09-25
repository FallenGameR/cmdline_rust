mod date;

use anyhow::{Ok, Result};
use chrono::{Datelike, Local, NaiveDate, Weekday};
use clap::{arg, Command};
use date::{Date, Month, Year};

// Field sizes reflect choises in the chrono crate
#[derive(Debug)]
pub struct Config {
    today: NaiveDate,
    month: Month,
    year: Year,
    show_year: bool,
}

const LINE_WIDTH: usize = 22;

pub fn get_args() -> Result<Config> {
    // CLI arguments
    // We try to mimic 'wsl ncal 10 2023 -b'
    let mut matches = Command::new("cal")
        .version("1.0")
        .author("FallenGameR")
        .about("Proleptic Gregorian month calendar with coloring")
        .args([
            arg!([DATE]... "Year number (1-9999) or month followed by year number")
                .help_heading("DATE as [[month] year]")
                .value_parser(Date::parse),
            arg!(-m --month <MONTH> "Month name or number (1-12)\nIs ignored if DATE specifies month")
                .value_parser(Date::parse_month),
            arg!(-y --show_year "Show calendar for the whole year")
                .conflicts_with("month"),
        ])
        .get_matches();

    // Parse arguments
    let today = Local::now().naive_local();
    let date: Option<Date> = matches.remove_one("DATE");
    let month: Option<Month> = matches.remove_one("month");

    // Construct config
    Ok(Config {
        today: today.date(),
        month: date
            .map(|d| d.month)
            .flatten()
            .or(month)
            .unwrap_or(Month(today.month())),
        year: date.map(|d| d.year).unwrap_or(Year(today.year())),
        show_year: matches.get_flag("show_year"),
    })
}

pub fn run(config: Config) -> Result<()> {
    dbg!(config);
    Ok(())
}

fn last_day_in_month(year: i32, month: u32) -> NaiveDate {
    const MIN_DAYS_IN_SHORTEST_MONTH: u32 = 28;
    let mut date = NaiveDate::from_ymd_opt(year, month, MIN_DAYS_IN_SHORTEST_MONTH)
        .expect("Date must be valid");
    let orig_month0 = date.month0();

    loop {
        date = date.succ_opt().expect("Date must be valid");
        if date.month0() != orig_month0 {
            return date.pred_opt().expect("Date must be valid");
        }
    }
}

// [0] "   February 2020      ",
// [1] "Su Mo Tu We Th Fr Sa  ",
// [2] "                   1  ",
// [3] " 2  3  4  5  6  7  8  ",
// [4] " 9 10 11 12 13 14 15  ",
// [5] "16 17 18 19 20 21 22  ",
// [6] "23 24 25 26 27 28 29  ",
// [7] "                      ",
//
// Plus current date needs to be highlighted
fn format_month(year: i32, month: u32, add_year: bool, _today: NaiveDate) -> Vec<String> {
    let mut result = Vec::new();
    let mut date = NaiveDate::from_ymd_opt(year, month, 1).expect("Date must be valid");

    // Header
    let month_text = date::MONTH_NAMES
        .get(date.month0() as usize)
        .expect("Date must be valid");
    let header_text = if add_year {
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

            let text = format!("{:2} ", date.day0() + 1);
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
    use super::{format_month, last_day_in_month};
    use chrono::NaiveDate;

    #[test]
    fn test_last_day_in_month() {
        assert_eq!(
            last_day_in_month(2020, 1),
            NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 2),
            NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()
        );
        assert_eq!(
            last_day_in_month(2020, 4),
            NaiveDate::from_ymd_opt(2020, 4, 30).unwrap()
        );
    }

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
