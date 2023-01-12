use maiq_parser::Fetch;
use rocket::{
  http::Status,
  request::{FromParam, FromRequest, Outcome},
  Request,
};

use crate::env;

use self::error::ApiError;

pub mod error;
pub mod routes;
mod utils;

#[derive(Debug, Clone)]
pub enum FetchParam {
  Today,
  Next,
}

impl<'a> FromParam<'a> for FetchParam {
  type Error = ApiError;

  fn from_param(param: &'a str) -> Result<Self, Self::Error> {
    match param {
      "today" => Ok(FetchParam::Today),
      "tomorrow" | "next" => Ok(FetchParam::Next),
      _ => Err(ApiError::SnapshotNotFound(param.to_string())),
    }
  }
}

impl Into<Fetch> for FetchParam {
  fn into(self) -> Fetch {
    match self {
      FetchParam::Today => Fetch::Today,
      FetchParam::Next => Fetch::Next,
    }
  }
}

impl ToString for FetchParam {
  fn to_string(&self) -> String {
    match self {
      FetchParam::Today => "today".into(),
      FetchParam::Next => "next".into(),
    }
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
