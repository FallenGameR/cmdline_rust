use anyhow::{bail, Ok, Result};
use chrono::{Datelike, Local, NaiveDate};
use clap::{arg, Command};

#[derive(Debug, Clone, Copy)]
struct Month(u32);

#[derive(Debug, Clone, Copy)]
struct Year(i32);

#[derive(Debug, Clone, Copy)]
struct Date(Year, Option<Month>);

// Field sizes reflect choises in the chrono crate
#[derive(Debug)]
pub struct Config {
    today: NaiveDate,
    month: Month,
    year: Year,
    show_year: bool,
}

const LINE_WIDTH: usize = 22;

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

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
                .value_parser(parse_date),
            arg!(-m --month <MONTH> "Month name or number (1-12)\nIs ignored if DATE specifies month")
                .value_parser(parse_month),
            arg!(-y --show_year "Show calendar for the whole year")
                .conflicts_with("month"),
        ])
        .get_matches();

    let today = Local::now().naive_local();
    let date: Option<Date> = matches.remove_one("DATE");

    // Construct config
    Ok(Config {
        today: today.date(),
        month: matches.remove_one("month").unwrap_or(Month(today.month())),
        year: date.map(|d| d.0).unwrap_or(Year(today.year())),
        show_year: matches.get_flag("show_year"),
    })
}

fn parse_month(month_text: &str) -> Result<Month> {
    let month_text = month_text.to_lowercase();
    let month_index = MONTH_NAMES
        .iter()
        .position(|&m| m.to_lowercase().starts_with(&month_text));

    let month = match month_index {
        Some(index) => (index + 1) as u32,
        None => month_text.parse::<u32>()?,
    };

    let allowed = 1..=12;
    if !allowed.contains(&month) {
        bail!(
            "month {month} not in the range [{},{}]",
            allowed.start(),
            allowed.end()
        );
    }

    Ok(Month(month))
}

fn parse_year(year_text: &str) -> Result<Year> {
    let year = year_text.parse::<i32>()?;
    let allowed = 1..=9999;

    if !allowed.contains(&year) {
        bail!(
            "year {year} not in the range [{},{}]",
            allowed.start(),
            allowed.end()
        );
    }

    Ok(Year(year))
}

fn parse_date(date_text: &str) -> Result<Date> {
    let parts = date_text.split(' ').collect::<Vec<_>>();
    match parts.as_slice() {
        [year] => Ok(Date(parse_year(year)?, None)),
        [month, year] => Ok(Date(parse_year(year)?, Some(parse_month(month)?))),
        _ => bail!("invalid date format")
    }
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
    use super::{format_month, last_day_in_month, parse_month, parse_year};
    use chrono::NaiveDate;

    #[test]
    fn test_parse_year() {
        let res = parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 1i32);

        let res = parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 9999i32);

        let res = parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year 0 not in the range [1,9999]"
        );

        let res = parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year 10000 not in the range [1,9999]"
        );

        let res = parse_year("foo");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "invalid digit found in string"
        );
    }

    #[test]
    fn test_parse_month() {
        let res = parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 1u32);

        let res = parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 12u32);

        let res = parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 1u32);

        let res = parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month 0 not in the range [1,12]"
        );

        let res = parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month 13 not in the range [1,12]"
        );

        let res = parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid digit found in string");
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
