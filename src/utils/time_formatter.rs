use jiff::{Timestamp, fmt::{rfc2822, strtime}, tz::TimeZone};

pub fn unix_to_custom_time(iso: i64) -> String {
    let timestamp = Timestamp::from_second(iso).unwrap();
    let zoned = timestamp.to_zoned(TimeZone::system());
    zoned.strftime("%a, %e %b %Y %I:%M %p UTC%z").to_string()
}

pub fn custom_time_to_unix(custom: &str) -> i64 {
    let zoned = strtime::parse("%a, %e %b %Y %I:%M %p UTC%z", custom)
        .unwrap()
        .to_zoned()
        .unwrap();
    zoned.timestamp().as_second()
}

pub fn rfc2822_to_custom(datetime: String) -> String {
    let zoned = rfc2822::parse(&datetime).unwrap();
    zoned.strftime("%a, %e %b %Y %I:%M %p UTC%z").to_string()
}

