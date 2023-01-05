use std::sync::Arc;

use maiq_parser::timetable::Snapshot;
use rocket::{http::Status, serde::json::Json, State};
use tokio::sync::Mutex;

use crate::{
  api::{FetchParam, TinySnapshot},
  cache::{CachePool, PollModel},
  db::{self, MongoPool},
};

use super::error::{ApiError, CustomApiError};

#[get("/")]
pub async fn index() -> Result<CustomApiError, ApiError> {
  Ok(CustomApiError { cause: "index_route", desc: "Hey there, stranger".into(), status: Status::Ok })
}

#[get("/latest/<fetch>")]
pub async fn latest(fetch: FetchParam, mongo: &State<MongoPool>) -> Result<Json<Snapshot>, ApiError> {
  match fetch {
    FetchParam::Today => db::get_latest_today(&mongo).await?,
    FetchParam::Tomorrow => db::get_latest_next(&mongo).await?,
  }
  .map(|s| Json(s))
  .ok_or(ApiError::SnapshotNotFound(fetch.to_string()))
}

#[get("/latest/<fetch>/<group>")]
pub async fn latest_group<'g>(
  fetch: FetchParam,
  group: &'g str,
  mongo: &State<MongoPool>,
) -> Result<Json<TinySnapshot>, ApiError> {
  match fetch {
    FetchParam::Today => db::get_latest_today(&mongo).await?,
    FetchParam::Tomorrow => db::get_latest_next(&mongo).await?,
  }
  .map(|s| Json(TinySnapshot::new_from_snapshot(group, &s)))
  .ok_or(ApiError::SnapshotNotFound(fetch.to_string()))
}

#[get("/poll")]
pub async fn poll(cache: &State<Arc<Mutex<CachePool>>>) -> Result<Json<PollModel>, ApiError> {
  Ok(Json(cache.lock().await.poll()))
}

#[get("/snapshot/<uid>")]
pub async fn snapshot_by_id<'a>(uid: &'a str, mongo: &State<MongoPool>) -> Result<Json<Snapshot>, ApiError> {
  db::get_by_uid(&mongo, uid)
    .await?
    .map(|s| Json(s))
    .ok_or(ApiError::SnapshotNotFound(uid.into()))
}
