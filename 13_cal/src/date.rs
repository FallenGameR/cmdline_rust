use anyhow::{bail, Ok, Result};

#[derive(Debug, Clone, Copy)]
pub struct Month(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Year(pub i32);

#[derive(Debug, Clone, Copy)]
pub struct Date {
    pub year: Year,
    pub month: Option<Month>
}

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

impl Date {
    pub fn new(date_text: &str) -> Result<Date> {
        let parts = date_text.split(' ').collect::<Vec<_>>();
        match parts.as_slice() {
            [year] => Date::from_year(year),
            [month, year] => Date::from_year_month(year, month),
            _ => bail!("invalid date format")
        }
    }

    fn from_year(year: &str) -> Result<Date> {
        Ok(Date{
            year: Date::parse_year(year)?,
            month: None,
        })
    }

    fn from_year_month(year: &str, month: &str) -> Result<Date> {
        Ok(Date{
            year: Date::parse_year(year)?,
            month: Some(Date::parse_month(month)?),
        })
    }

    pub fn parse_month(month_text: &str) -> Result<Month> {
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
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::Date;

    #[test]
    fn test_parse_year() {
        let res = Date::parse_year("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 1i32);

        let res = Date::parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 9999i32);

        let res = Date::parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year 0 not in the range [1,9999]"
        );

        let res = Date::parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year 10000 not in the range [1,9999]"
        );

        let res = Date::parse_year("foo");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "invalid digit found in string"
        );
    }

    #[test]
    fn test_parse_month() {
        let res = Date::parse_month("1");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 1u32);

        let res = Date::parse_month("12");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 12u32);

        let res = Date::parse_month("jan");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 1u32);

        let res = Date::parse_month("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month 0 not in the range [1,12]"
        );

        let res = Date::parse_month("13");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "month 13 not in the range [1,12]"
        );

        let res = Date::parse_month("foo");
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().to_string(), "invalid digit found in string");
    }
}