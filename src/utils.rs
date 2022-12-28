use chrono::{DateTime, Days, Duration, Timelike, Utc};

pub fn current_date(offset: u64) -> DateTime<Utc> {
  let now = Utc::now() + Duration::hours(3);
  now
    .with_hour(0)
    .unwrap()
    .with_minute(0)
    .unwrap()
    .with_second(0)
    .unwrap()
    .with_nanosecond(0)
    .unwrap()
    .checked_add_days(Days::new(offset))
    .unwrap()
}
