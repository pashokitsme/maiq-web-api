use chrono::{Days, Duration, Utc};

pub fn date_timestamp(offset: u64) -> i64 {
  let now = Utc::now().naive_utc() + Duration::seconds(60 * 60 * 3);
  now
    .date()
    .checked_add_days(Days::new(offset))
    .unwrap()
    .and_hms_opt(0, 0, 0)
    .unwrap()
    .timestamp()
}
