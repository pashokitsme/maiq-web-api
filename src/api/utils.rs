use chrono::Weekday;

pub fn map_weekday<'a>(weekday: &'a str) -> Option<Weekday> {
  let day = match weekday.to_lowercase().as_str() {
    "mon" => Weekday::Mon,
    "tue" => Weekday::Tue,
    "wed" => Weekday::Wed,
    "thu" => Weekday::Thu,
    "fri" => Weekday::Fri,
    "sat" => Weekday::Sat,
    _ => return None,
  };
  Some(day)
}
