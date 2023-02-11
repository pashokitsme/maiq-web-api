use std::ops::Deref;
use std::sync::Arc;

use chrono::Weekday;
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
      _ => Err(ApiError::SnapshotNotFound(param.to_string())),
    }
  }
}

impl Deref for FetchParam {
  type Target = Fetch;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

pub struct ApiKey;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for ApiKey {
  type Error = ApiError;

  async fn from_request(req: &'r Request<'_>) -> Outcome<ApiKey, Self::Error> {
    fn is_valid(key: &str) -> bool {
      lazy_static::lazy_static! {
        static ref RIGHT_KEY: String = env::var(env::API_SECRET).unwrap();
      }
      key == *RIGHT_KEY
    }

    match req.headers().get_one("x-api-key") {
      None => Outcome::Failure((Status::Unauthorized, ApiError::InvalidApiKey)),
      Some(key) if is_valid(key) => Outcome::Success(ApiKey),
      Some(_) => Outcome::Failure((Status::Unauthorized, ApiError::InvalidApiKey)),
    }
  }
}

pub fn map_weekday<'a>(weekday: &'a str) -> Option<Weekday> {
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
