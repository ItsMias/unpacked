//! Minimal UTC timestamp handling (no chrono dependency, keeps the WASM small).

/// Parses `YYYY-MM-DD[T ]HH:MM:SS...` into Unix milliseconds (UTC).
/// Tolerates the stray quotes Discord wraps event timestamps in (`"\"2025-07-04T11:27:56Z\""`).
pub fn parse_ms(s: &str) -> Option<i64> {
    let s = s.trim_matches(|c| c == '"' || c == '\\');
    if s.len() < 19 {
        return None;
    }
    let n = |a: usize, b: usize| s.get(a..b)?.parse::<i64>().ok();
    let (y, mo, d) = (n(0, 4)?, n(5, 7)?, n(8, 10)?);
    let (h, mi, se) = (n(11, 13)?, n(14, 16)?, n(17, 19)?);
    let secs = days_from_civil(y, mo, d) * 86_400 + h * 3600 + mi * 60 + se;
    Some(secs * 1000)
}

/// Days since 1970-01-01 (Howard Hinnant's algorithm).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Unix ms at the start of the given UTC date.
pub fn date_ms(y: i64, m: i64, d: i64) -> i64 {
    days_from_civil(y, m, d) * 86_400_000
}

/// (year, month 1..=12, day) of a day number since 1970-01-01 (Howard Hinnant's algorithm).
pub fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Month index 0..=11 of a Unix ms timestamp.
pub fn month_of(ms: i64) -> usize {
    (civil_from_days(ms.div_euclid(DAY_MS)).1 - 1) as usize
}

pub const HOUR_MS: i64 = 3_600_000;
pub const DAY_MS: i64 = 86_400_000;

/// Months since January 2015 (Discord's launch year): the bucket size for per-channel counts.
pub fn month_index(ms: i64) -> u16 {
    let (y, m, _) = civil_from_days(ms.div_euclid(DAY_MS));
    ((y - 2015) * 12 + m - 1).clamp(0, u16::MAX as i64) as u16
}

/// Hours since the Unix epoch: the bucket size for message counts.
pub fn hour_index(ms: i64) -> u32 {
    ms.div_euclid(HOUR_MS).clamp(0, u32::MAX as i64) as u32
}

/// Creation time of a Discord snowflake id, in Unix ms.
pub fn snowflake_ms(id: u64) -> i64 {
    (id >> 22) as i64 + 1_420_070_400_000
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_formats() {
        let a = parse_ms("\"2025-07-04T11:27:56Z\"").unwrap();
        let b = parse_ms("2025-07-04 11:27:56").unwrap();
        assert_eq!(a, b);
        assert_eq!(a, 1_751_628_476_000);
        assert_eq!(parse_ms("\"2025-07-04T11:27:56.123Z\""), Some(a));
        assert_eq!(parse_ms("nope"), None);
    }

    #[test]
    fn calendar_round_trip() {
        for (y, m, d) in [(1970, 1, 1), (2019, 8, 26), (2024, 2, 29), (2026, 12, 31)] {
            assert_eq!(civil_from_days(days_from_civil(y, m, d)), (y, m, d));
        }
        assert_eq!(month_index(date_ms(2015, 1, 1)), 0);
        assert_eq!(month_index(date_ms(2024, 8, 1)), 9 * 12 + 7);
        // The example id from Discord's API docs.
        assert_eq!(snowflake_ms(175928847299117063), 1_462_015_105_796);
    }

    #[test]
    fn months() {
        assert_eq!(month_of(date_ms(2025, 1, 1)), 0);
        assert_eq!(month_of(date_ms(2025, 12, 31)), 11);
        assert_eq!(month_of(date_ms(2024, 2, 29)), 1);
    }
}
