use chrono::{NaiveDateTime, Utc};
use chrono_tz::Europe::Moscow;

pub fn get_current_datetime() -> NaiveDateTime {
    Utc::now().with_timezone(&Moscow).naive_utc()
}
