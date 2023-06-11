use std::ops::Deref;
use std::sync::Arc;

use self::error::ApiError;
use crate::cache;
use chrono::Weekday;
use maiq_parser::Fetch;
use rocket::{request::FromParam, State};
use tokio::sync::RwLock;

pub mod error;
pub mod routes;

type CachePool = State<Arc<RwLock<cache::CachePool>>>;

#[derive(Debug)]
pub struct FetchParam(Fetch);

impl FromParam<'_> for FetchParam {
  type Error = ApiError;

  fn from_param(param: &str) -> Result<Self, Self::Error> {
    match param {
      "today" => Ok(FetchParam(Fetch::Today)),
      "tomorrow" | "next" => Ok(FetchParam(Fetch::Next)),
      _ => Err(ApiError::InvalidQueryParam(param.to_string())),
    }
  }
}

impl Deref for FetchParam {
  type Target = Fetch;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub fn map_weekday(weekday: &str) -> Option<Weekday> {
  let day = match weekday.to_lowercase().as_str() {
    "mon" => Weekday::Mon,
    "tue" => Weekday::Tue,
    "wed" => Weekday::Wed,
    "thu" => Weekday::Thu,
    "fri" => Weekday::Fri,
    "sat" => Weekday::Sat,
    "sun" => Weekday::Sun,
    _ => return None,
  };
  Some(day)
}
