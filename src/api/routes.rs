use std::sync::Arc;

use maiq_parser::timetable::Snapshot;
use rocket::{http::Status, serde::json::Json, State};
use tokio::sync::Mutex;

use crate::{
  api::{FetchParam, TinySnapshot},
  cache::{CachePool, Poll},
  db::{self, MongoPool},
};

use super::error::{ApiError, CustomApiError};

#[get("/")]
pub async fn index() -> Result<CustomApiError, ApiError> {
  Ok(CustomApiError { cause: "index_route", desc: "Hey there, stranger".into(), status: Status::Ok })
}

#[get("/latest/<fetch>")]
pub async fn latest(
  fetch: FetchParam,
  mongo: &State<MongoPool>,
  cache: &State<Arc<Mutex<CachePool>>>,
) -> Result<Json<Snapshot>, ApiError> {
  if let Some(s) = cache.lock().await.cached(fetch.clone().into()).await {
    info!("Found cached {}!", s.uid);
    return Ok(Json(s));
  }

  info!("Trying to fetch {:?} snapshot from db", fetch);
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
  cache: &State<Arc<Mutex<CachePool>>>,
) -> Result<Json<TinySnapshot>, ApiError> {
  if let Some(s) = cache.lock().await.cached(fetch.clone().into()).await {
    info!("Found cached {}!", s.uid);
    return Ok(Json(TinySnapshot::new_from_snapshot(group, &s)));
  }

  info!("Trying to fetch {:?} snapshot from db", fetch);
  match fetch {
    FetchParam::Today => db::get_latest_today(&mongo).await?,
    FetchParam::Tomorrow => db::get_latest_next(&mongo).await?,
  }
  .map(|s| Json(TinySnapshot::new_from_snapshot(group, &s)))
  .ok_or(ApiError::SnapshotNotFound(fetch.to_string()))
}

#[get("/poll")]
pub async fn poll(cache: &State<Arc<Mutex<CachePool>>>) -> Result<Json<Poll>, ApiError> {
  Ok(Json(cache.lock().await.poll()))
}

#[get("/snapshot/<uid>")]
pub async fn snapshot_by_id<'a>(
  uid: &'a str,
  mongo: &State<MongoPool>,
  cache: &State<Arc<Mutex<CachePool>>>,
) -> Result<Json<Snapshot>, ApiError> {
  if let Some(s) = cache.lock().await.cached_by_uid(uid).await {
    info!("Found cached {}!", s.uid);
    return Ok(Json(s));
  }
  info!("Trying to fetch snapshot {} from db", uid);
  db::get_by_uid(&mongo, uid)
    .await?
    .map(|s| Json(s))
    .ok_or(ApiError::SnapshotNotFound(uid.into()))
}
