#![cfg(feature = "reqwest")]

use std::env;

use chrono::{NaiveDate, Weekday};
use lazy_static::lazy_static;
use maiq_shared::{default::DefaultGroup, Fetch, Snapshot};
use reqwest::StatusCode;
use serde::{de::DeserializeOwned, Deserialize};

use crate::polling::Poll;

#[derive(Deserialize, Debug)]
pub struct ApiError {
  pub cause: String,
  pub desc: String,
}

lazy_static! {
  static ref API_HOST: String = env::var("API_HOST").expect("API host not set");
  static ref TODAY_URL: String = format!("{}/latest/today", *API_HOST);
  static ref NEXT_URL: String = format!("{}/latest/next", *API_HOST);
  static ref POLL_URL: String = format!("{}/poll", *API_HOST);
}

impl From<reqwest::Error> for ApiError {
  fn from(e: reqwest::Error) -> Self {
    ApiError { cause: "reqwest".into(), desc: e.to_string() }
  }
}

pub async fn latest(fetch: Fetch) -> Result<Snapshot, ApiError> {
  match fetch {
    Fetch::Today => get(&*TODAY_URL).await,
    Fetch::Next => get(&*NEXT_URL).await,
  }
}

pub async fn snapshot(uid: &str) -> Result<Snapshot, ApiError> {
  get(&format!("{}/uid/{}", *API_HOST, uid)).await
}

pub async fn date(date: NaiveDate) -> Result<Snapshot, ApiError> {
  get(&format!("{}/date/{}", *API_HOST, date.format("%d.%m.%Y"))).await
}

pub async fn default(group: &str, weekday: Weekday) -> Result<DefaultGroup, ApiError> {
  get(&format!("{}/default/{}/{}", *API_HOST, weekday, group)).await
}

pub async fn poll() -> Result<Poll, ApiError> {
  get(&*POLL_URL).await
}

async fn get<O: DeserializeOwned>(url: &str) -> Result<O, ApiError> {
  let res = reqwest::get(url).await?;
  match res.status() {
    StatusCode::OK => Ok(res.json().await?),
    _ => Err(res.json().await?),
  }
}
