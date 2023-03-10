use std::ops::Deref;
use std::sync::Arc;

use chrono::Weekday;
use mongodb::bson::DateTime;
use rocket::{
  http::Status,
  request::{FromParam, FromRequest, Outcome},
  Request, State,
};

use tokio::sync::RwLock;

use crate::{
  env,
  storage::{cache, mongo},
};
use maiq_parser::Fetch;

use self::error::ApiError;

pub mod error;
pub mod routes;

type CachePool = State<Arc<RwLock<cache::CachePool>>>;
type MongoPool = State<mongo::MongoPool>;

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

pub struct DateParam(DateTime);

impl FromParam<'_> for DateParam {
  type Error = ApiError;

  fn from_param(param: &str) -> Result<Self, Self::Error> {
    let err = || ApiError::InvalidQueryParam(param.into());
    let parse = |x: &str| x.parse().ok();
    let mut slice = param.split('.');
    let d = slice.next().and_then(parse).ok_or_else(err)?;
    let m = slice.next().and_then(parse).ok_or_else(err)?;
    let y = slice.next().and_then(|y| y.parse::<i32>().ok()).ok_or_else(err)?;

    let date = DateTime::builder().day(d).month(m).year(y).build();
    Ok(DateParam(date.map_err(|_| err())?))
  }
}

pub struct ApiKey;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
  type Error = ApiError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<ApiKey, Self::Error> {
    fn is_valid(key: &str) -> bool {
      lazy_static::lazy_static! {
        static ref KEY: String = env::var(env::API_SECRET).unwrap();
      }
      key == *KEY
    }

    match req.headers().get_one("x-api-key") {
      None => Outcome::Failure((Status::Unauthorized, ApiError::InvalidApiKey)),
      Some(key) if is_valid(key) => Outcome::Success(ApiKey),
      Some(_) => Outcome::Failure((Status::Unauthorized, ApiError::InvalidApiKey)),
    }
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
