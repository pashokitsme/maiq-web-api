use maiq_api_wrapper::Poll;
use maiq_parser::{default::DefaultGroup, Snapshot, TinySnapshot};
use rocket::{http::Status, serde::json::Json};

use crate::{
  api::{map_weekday, CachePool, FetchParam},
  cache::SnapshotPool,
};

use super::error::{ApiError, CustomApiError};

#[get("/")]
pub fn index() -> Result<CustomApiError, ApiError> {
  Ok(CustomApiError { cause: "index_route", desc: "Hey there, stranger".into(), status: Status::Ok })
}

#[get("/groups")]
pub fn groups() -> Json<Vec<String>> {
  Json(maiq_parser::env::groups().into())
}

#[get("/default/<weekday>/<group>")]
pub fn default<'a>(weekday: &str, group: &'a str) -> Result<Json<&'a DefaultGroup>, ApiError> {
  let not_found = || ApiError::DefaultNotFound(weekday.into(), group.into());
  let weekday = map_weekday(weekday).ok_or_else(not_found)?;
  maiq_parser::default_for(weekday, group)
    .map(Json)
    .ok_or_else(not_found)
}

#[get("/latest/<fetch>")]
pub async fn latest(fetch: FetchParam, cache: &CachePool) -> Result<Json<Snapshot>, ApiError> {
  let fetch = *fetch;
  cache.read().await.latest(fetch).await?.ok_or(fetch.into()).map(Json)
}

#[get("/latest/<fetch>/<group>")]
pub async fn latest_group(fetch: FetchParam, group: &str, cache: &CachePool) -> Result<Json<TinySnapshot>, ApiError> {
  let fetch = *fetch;
  cache
    .read()
    .await
    .latest(fetch)
    .await?
    .ok_or(fetch.into())
    .map(|s| Json(s.tiny(group)))
}

#[get("/poll")]
pub async fn poll(cache: &CachePool) -> Result<Json<Poll>, ApiError> {
  Ok(Json(cache.read().await.poll()))
}

#[get("/uid/<uid>")]
pub async fn snapshot_by_id(uid: &str, cache: &CachePool) -> Result<Json<Snapshot>, ApiError> {
  cache
    .read()
    .await
    .by_uid(uid)
    .await?
    .ok_or(ApiError::SnapshotNotFound(uid.into()))
    .map(Json)
}
