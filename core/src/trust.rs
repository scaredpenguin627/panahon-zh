//! Validation and time semantics shared by the adapters and presentation.
use chrono::{Datelike, Duration, FixedOffset, NaiveDate, NaiveDateTime, TimeZone, Utc};

pub fn now_epoch() -> i64 {
    Utc::now().timestamp()
}

pub fn manila_today() -> NaiveDate {
    (Utc::now() + Duration::hours(8)).date_naive()
}

/// The public Philippine feeds use local UTC+08 times when no offset is present.
pub fn timestamp(raw: &str) -> Option<i64> {
    if let Ok(value) = chrono::DateTime::parse_from_rfc3339(raw) {
        return Some(value.timestamp());
    }
    let zone = FixedOffset::east_opt(8 * 3600)?;
    for format in ["%Y-%m-%d %H:%M:%S", "%Y-%m-%dT%H:%M", "%B %d, %Y %I:%M %p"] {
        if let Ok(value) = NaiveDateTime::parse_from_str(raw.trim(), format) {
            return zone
                .from_local_datetime(&value)
                .single()
                .map(|date| date.timestamp());
        }
    }
    None
}

pub fn fresh_observation(raw: &str, now: i64) -> bool {
    timestamp(raw).is_some_and(|at| (-900..=3 * 3600).contains(&(now - at)))
}

pub fn forecast_date(raw: &str, label: &str, today: NaiveDate) -> String {
    if let Some(date) = raw
        .get(..10)
        .and_then(|date| NaiveDate::parse_from_str(date, "%Y-%m-%d").ok())
    {
        return date.to_string();
    }
    let parts: Vec<_> = label.split('/').collect();
    if parts.len() == 2 {
        if let (Ok(month), Ok(day)) = (parts[0].parse(), parts[1].parse()) {
            // Dec/Jan feeds may span two years. Choose the closest valid date.
            if let Some(date) = ((today.year() - 1)..=(today.year() + 1))
                .filter_map(|year| NaiveDate::from_ymd_opt(year, month, day))
                .min_by_key(|date| (*date - today).num_days().abs())
            {
                return date.to_string();
            }
        }
    }
    String::new()
}

pub fn day_label(raw: &str, english: bool) -> String {
    let Ok(date) = NaiveDate::parse_from_str(raw, "%Y-%m-%d") else {
        return raw.to_owned();
    };
    let delta = (date - manila_today()).num_days();
    match (delta, english) {
        (0, true) => "Today".into(),
        (0, false) => "今天".into(),
        (1, true) => "Tomorrow".into(),
        (1, false) => "明天".into(),
        (_, true) => date.format("%a").to_string(),
        (_, false) => ["周一", "周二", "周三", "周四", "周五", "周六", "周日"]
            [date.weekday().num_days_from_monday() as usize]
            .into(),
    }
}

pub fn clock_label(raw: &str) -> String {
    for format in ["%I:%M %p", "%H:%M", "%H:%M:%S"] {
        if let Ok(time) =
            chrono::NaiveTime::parse_from_str(&raw.trim().to_ascii_uppercase(), format)
        {
            return time.format("%H:%M").to_string();
        }
    }
    "—".into()
}

/// Parse a single numeric value plus an allowed unit, never a range or probability.
pub fn measurement(raw: &str) -> Option<f64> {
    let raw = raw.trim();
    let end = raw
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit() && !matches!(c, '+' | '-' | '.'))
        .map(|(i, _)| i)
        .unwrap_or(raw.len());
    let number: f64 = raw.get(..end)?.parse().ok()?;
    let suffix = raw.get(end..)?.trim().to_ascii_lowercase();
    if ![
        "", "°c", "c", "%", "km/h", "kph", "mm", "mm/hr", "mm/h", "hpa", "w/m²", "w/m2",
    ]
    .contains(&suffix.as_str())
    {
        return None;
    }
    number.is_finite().then_some(number)
}

pub fn heat_band(celsius: f32) -> &'static str {
    if celsius >= 52.0 {
        "极度危险"
    } else if celsius >= 42.0 {
        "危险"
    } else if celsius >= 33.0 {
        "高度警惕"
    } else if celsius >= 27.0 {
        "注意"
    } else {
        "低于分级范围"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn malformed_values_do_not_turn_into_weather() {
        for raw in [
            "<0.1 mm",
            "10-20 mm",
            "NaN",
            "- hPa",
            "rain 2 mm",
            "3 hours",
            "1e999",
        ] {
            assert_eq!(measurement(raw), None, "{raw}");
        }
        assert_eq!(measurement("-2.5 °C"), Some(-2.5));
        assert_eq!(measurement("0 mm/hr"), Some(0.0));
    }
    #[test]
    fn date_rollover_is_absolute_and_invalid_dates_stay_empty() {
        let today = NaiveDate::from_ymd_opt(2026, 12, 31).unwrap();
        assert_eq!(forecast_date("Fri", "01/01", today), "2027-01-01");
        assert_eq!(forecast_date("Today", "02/30", today), "");
        assert_eq!(
            forecast_date("2026-11-12T00:00:00Z", "", today),
            "2026-11-12"
        );
    }
    #[test]
    fn clock_never_invents_now_or_midnight() {
        assert_eq!(clock_label("12:00 AM"), "00:00");
        assert_eq!(clock_label("12:00 PM"), "12:00");
        assert_eq!(clock_label("bad"), "—");
        assert_eq!(clock_label("9:00 AM"), "09:00");
    }
    #[test]
    fn heat_thresholds_use_pagasa_bands() {
        for (input, expected) in [
            (26., "低于分级范围"),
            (27., "注意"),
            (32., "注意"),
            (33., "高度警惕"),
            (41., "高度警惕"),
            (42., "危险"),
            (51., "危险"),
            (52., "极度危险"),
        ] {
            assert_eq!(heat_band(input), expected);
        }
    }
    #[test]
    fn stale_future_and_unparseable_observations_are_rejected() {
        let now = timestamp("2026-11-12 09:00:00").unwrap();
        assert!(fresh_observation("2026-11-12 08:00:00", now));
        assert!(!fresh_observation("2026-11-10 08:00:00", now));
        assert!(!fresh_observation("2026-11-13 08:00:00", now));
        assert!(!fresh_observation("unknown", now));
    }
}
