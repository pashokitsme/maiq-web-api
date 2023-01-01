use chrono::{DateTime, Utc};
use maiq_parser::{Fetch, Group, Snapshot};
use rocket::request::FromParam;
use serde::Serialize;

use self::error::ApiError;

pub mod error;
pub mod routes;

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
      _ => Err(ApiError::TimetableNotFound(param.to_string())),
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

impl ToString for FetchParam {
  fn to_string(&self) -> String {
    match self {
      FetchParam::Today => "today".into(),
      FetchParam::Tomorrow => "next".into(),
    }
  }
}

#[derive(Serialize)]
pub struct TinySnapshot {
  pub uid: String,
  pub date: DateTime<Utc>,
  pub parsed_date: DateTime<Utc>,
  pub group: Option<Group>,
}

impl TinySnapshot {
  pub fn new_from_snapshot<'a>(name: &'a str, snapshot: &Snapshot) -> Self {
    let group = snapshot
      .groups
      .iter()
      .find(|g| g.name == name)
      .and_then(|g| Some(g.to_owned()));

    Self { uid: snapshot.uid.clone(), date: snapshot.date, parsed_date: snapshot.parsed_date, group }
  }
}
