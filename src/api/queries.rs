use maiq_parser::Fetch;
use rocket::request::FromParam;

use super::error::ApiError;

pub enum FetchParam {
  Today,
  Tomorrow,
}

impl<'a> FromParam<'a> for FetchParam {
  type Error = ApiError;

  fn from_param(param: &'a str) -> Result<Self, Self::Error> {
    match param {
      "today" => Ok(FetchParam::Today),
      "tomorrow" | "next" => Ok(FetchParam::Tomorrow),
      _ => Err(ApiError::ResourseNotFound(param.to_string())),
    }
  }
}

impl Into<Fetch> for FetchParam {
  fn into(self) -> Fetch {
    match self {
      FetchParam::Today => Fetch::Today,
      FetchParam::Tomorrow => Fetch::Tomorrow,
    }
  }
}
