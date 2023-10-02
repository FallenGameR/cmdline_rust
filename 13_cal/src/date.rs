use anyhow::{bail, Ok, Result};

#[derive(Debug, Clone, Copy)]
pub struct Month(pub u32);

#[derive(Debug, Clone, Copy)]
pub struct Year(pub i32);

#[derive(Debug, Clone, Copy)]
pub struct Date {
    pub year: Year,
    pub month: Option<Month>,
}

pub const MONTH_NAMES: [&str; 12] = [
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
    /// Parse date from string:
    /// - YEAR
    /// - MONTH YEAR
    pub fn parse(date: &str) -> Result<Date> {
        let parts = date.split(' ').collect::<Vec<_>>();
        match parts.as_slice() {
            [year] => Ok(Date {
                year: Date::parse_year(year)?,
                month: None,
            }),
            [month, year] => Ok(Date {
                year: Date::parse_year(year)?,
                month: Some(Date::parse_month(month)?),
            }),
            _ => bail!("invalid date format"),
        }
    }

    pub fn parse_year(year_text: &str) -> Result<Year> {
        let year = year_text.parse::<i32>()?;
        let allowed = 99..=9999;

        if !allowed.contains(&year) {
            bail!(
                "year {year} not in the range [{},{}]",
                allowed.start(),
                allowed.end()
            );
        }

        Ok(Year(year))
    }

    pub fn parse_month(month_text: &str) -> Result<Month> {
        let month_text = month_text.to_lowercase();
        let month_index = MONTH_NAMES
            .iter()
            .position(|&m| m.to_lowercase().starts_with(&month_text));

        let month = match month_index {
            None => month_text.parse::<u32>()?,
            Some(index) => (index + 1) as u32,
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
}

// --------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::Date;

    #[test]
    fn test_parse_year() {
        let res = Date::parse_year("1");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year 1 not in the range [99,9999]"
        );

        let res = Date::parse_year("100");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 100i32);

        let res = Date::parse_year("9999");
        assert!(res.is_ok());
        assert_eq!(res.unwrap().0, 9999i32);

        let res = Date::parse_year("0");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year 0 not in the range [99,9999]"
        );

        let res = Date::parse_year("10000");
        assert!(res.is_err());
        assert_eq!(
            res.unwrap_err().to_string(),
            "year 10000 not in the range [99,9999]"
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
        assert_eq!(
            res.unwrap_err().to_string(),
            "invalid digit found in string"
        );
    }
}
