//! Reads the clock of design §3.6: MEMEX_NOW when it is set, else the system clock, and formats the instants that log files carry.
//! Cases: LOG-WRITE-01, LOG-WRITE-11

use std::ops::Range;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Error;

/// Seconds in a day, for splitting an instant into a day number and a time of day.
const DAY: i64 = 86_400;

/// An instant in UTC, as whole seconds since 1970-01-01T00:00:00Z.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp(i64);

impl Timestamp {
    /// The RFC 3339 form in UTC, to the second: `2026-10-13T12:00:00Z`. This is the `time` field of a log.
    pub fn rfc3339(self) -> String {
        let (year, month, day) = self.date();
        let (hour, minute, second) = self.time_of_day();
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
    }

    /// The compact stamp that starts each log file name, in UTC: `20261013-120000`.
    pub fn stamp(self) -> String {
        let (year, month, day) = self.date();
        let (hour, minute, second) = self.time_of_day();
        format!("{year:04}{month:02}{day:02}-{hour:02}{minute:02}{second:02}")
    }

    /// The civil date (year, month, day) of this instant.
    fn date(self) -> (i64, i64, i64) {
        civil_from_days(self.0.div_euclid(DAY))
    }

    /// The time of day (hour, minute, second) of this instant.
    fn time_of_day(self) -> (i64, i64, i64) {
        let rest = self.0.rem_euclid(DAY);
        (rest / 3_600, rest % 3_600 / 60, rest % 60)
    }
}

/// The current instant: MEMEX_NOW when it is set, else the system clock. An invalid MEMEX_NOW is exit 3.
pub fn now() -> Result<Timestamp, Error> {
    let Some(raw) = std::env::var_os("MEMEX_NOW") else {
        return Ok(system_now());
    };
    let text = raw.to_string_lossy();
    parse(&text).ok_or_else(|| Error::io_message(format!("invalid MEMEX_NOW: {text}")))
}

/// The system clock. A clock set before 1970 gives a negative instant rather than a failure.
fn system_now() -> Timestamp {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(elapsed) => Timestamp(elapsed.as_secs() as i64),
        Err(before) => Timestamp(-(before.duration().as_secs() as i64)),
    }
}

/// Parses exactly the UTC form `YYYY-MM-DDTHH:MM:SSZ`. An offset, a fraction or any other spelling is refused.
fn parse(text: &str) -> Option<Timestamp> {
    let separators = [
        (4, b'-'),
        (7, b'-'),
        (10, b'T'),
        (13, b':'),
        (16, b':'),
        (19, b'Z'),
    ];
    let shaped = text.len() == 20
        && separators
            .iter()
            .all(|&(at, byte)| text.as_bytes()[at] == byte);
    if !shaped {
        return None;
    }
    let (year, month, day) = (
        digits(text, 0..4)?,
        digits(text, 5..7)?,
        digits(text, 8..10)?,
    );
    let (hour, minute, second) = (
        digits(text, 11..13)?,
        digits(text, 14..16)?,
        digits(text, 17..19)?,
    );
    let valid = (1..=12).contains(&month)
        && (1..=days_in_month(year, month)).contains(&day)
        && hour <= 23
        && minute <= 59
        && second <= 59;
    valid.then(|| {
        let days = days_from_civil(year, month, day);
        Timestamp(days * DAY + hour * 3_600 + minute * 60 + second)
    })
}

/// The number written in `text` at `at`, or None when any byte there is not an ASCII digit.
fn digits(text: &str, at: Range<usize>) -> Option<i64> {
    let part = text.get(at)?;
    if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

/// The number of days in a month of a year, leap years included.
fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

/// True for a Gregorian leap year.
fn is_leap(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

/// Days from 1970-01-01 to a civil date, proleptic Gregorian (Howard Hinnant's days-from-civil).
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let shifted_month = (month + 9) % 12;
    let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

/// The civil date (year, month, day) of a day number from 1970-01-01 (Hinnant's civil-from-days).
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let days = days + 719_468;
    let era = days.div_euclid(146_097);
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_instants_convert_both_ways() {
        assert_eq!(days_from_civil(1970, 1, 1), 0);
        assert_eq!(days_from_civil(2000, 3, 1) * DAY, 951_868_800);
        assert_eq!(
            parse("2026-10-13T12:00:00Z"),
            Some(Timestamp(1_791_892_800))
        );
        assert_eq!(civil_from_days(-719_468), (0, 3, 1));
    }

    #[test]
    fn only_the_utc_form_parses() {
        let refused = [
            "2026-10-13T12:00:00+01:00",
            "2026-10-13 12:00:00Z",
            "2026-10-13T12:00:00.5Z",
            "2026-02-29T00:00:00Z",
            "2026-13-01T00:00:00Z",
            "2026-10-13T24:00:00Z",
            "2026-10-13T12:60:00Z",
            "not-a-time",
            "",
        ];
        for text in refused {
            assert_eq!(parse(text), None, "{text}");
        }
        assert!(parse("2024-02-29T23:59:59Z").is_some());
    }

    #[test]
    fn formats_match_the_design_examples() {
        let at = Timestamp(1_791_892_800);
        assert_eq!(at.rfc3339(), "2026-10-13T12:00:00Z");
        assert_eq!(at.stamp(), "20261013-120000");
    }

    #[test]
    fn instants_before_1970_round_trip() {
        let at = parse("1969-12-31T23:59:59Z").expect("a valid instant");
        assert_eq!(at, Timestamp(-1));
        assert_eq!(at.rfc3339(), "1969-12-31T23:59:59Z");
    }
}
