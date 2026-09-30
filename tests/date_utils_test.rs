use logmaster_batch_downloader::date_utils::{is_leap_year, last_day_of_month, DateError};

#[test]
fn test_leap_years() {
    assert!(is_leap_year(2020));
    assert!(is_leap_year(2024));
    assert!(is_leap_year(2000));
    assert!(!is_leap_year(2021));
    assert!(!is_leap_year(2023));
    assert!(!is_leap_year(1900));
    assert!(!is_leap_year(2100));
}

#[test]
fn test_last_day_of_month() {
    assert_eq!(last_day_of_month(2024, 2).unwrap(), 29);
    assert_eq!(last_day_of_month(2023, 2).unwrap(), 28);
    assert_eq!(last_day_of_month(2024, 1).unwrap(), 31);
    assert_eq!(last_day_of_month(2024, 4).unwrap(), 30);
    assert!(matches!(last_day_of_month(2024, 13), Err(DateError::InvalidMonth(13))));
    assert!(matches!(last_day_of_month(2024, 0), Err(DateError::InvalidMonth(0))));
}
