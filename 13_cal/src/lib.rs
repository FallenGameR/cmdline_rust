mod date;

use date::{Date, Month, Year};
use anyhow::{Ok, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::{arg, Command};

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
                .value_parser(Date::new),
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
        month: date.map(|d| d.month).flatten().or(month).unwrap_or(Month(today.month())),
        year: date.map(|d| d.year).unwrap_or(Year(today.year())),
        show_year: matches.get_flag("show_year"),
    })
}

pub fn run(config: Config) -> Result<()> {
    dbg!(config);
    Ok(())
}

fn format_month(_year: i32, _month: u32, _print_year: bool, _today: NaiveDate) -> Vec<String> {
    todo!()
}

fn last_day_in_month(_year: i32, _month: u32) -> NaiveDate {
    todo!()
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use super::{format_month, last_day_in_month};
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
}
