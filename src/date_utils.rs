use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DateError {
    #[error("Invalid month: {0}")]
    InvalidMonth(u32),
}

pub fn is_leap_year(year: u32) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub fn last_day_of_month(year: u32, month: u32) -> Result<u32, DateError> {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => Ok(31),
        4 | 6 | 9 | 11 => Ok(30),
        2 => {
            if is_leap_year(year) {
                Ok(29)
            } else {
                Ok(28)
            }
        }
        m => Err(DateError::InvalidMonth(m)),
    }
}
