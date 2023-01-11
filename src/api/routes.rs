use std::sync::Arc;

use maiq_parser::{default::DefaultGroup, Snapshot, TinySnapshot};
use rocket::{http::Status, serde::json::Json, State};
use tokio::sync::Mutex;

use crate::{
  api::FetchParam,
  cache::{CachePool, Poll},
  db::MongoPool,
};

use super::{
  error::{ApiError, CustomApiError},
  utils::map_weekday,
  ApiKey,
};

#[get("/")]
pub async fn index() -> Result<CustomApiError, ApiError> {
  Ok(CustomApiError { cause: "index_route", desc: "Hey there, stranger".into(), status: Status::Ok })
}

#[get("/default/<weekday>/<group>")]
pub fn default<'a>(weekday: &'a str, group: &'a str) -> Result<Json<DefaultGroup>, ApiError> {
  macro_rules! not_found {
    () => {
      ApiError::DefaultNotFound(weekday.into(), group.into())
    };
  }
  let repls = &*maiq_parser::replacer::REPLECEMENTS;
  let weekday = map_weekday(weekday).ok_or(not_found!())?;
  repls
    .iter()
    .find(|d| d.day == weekday)
    .ok_or(not_found!())?
    .groups
    .iter()
    .find(|g| g.name.as_str() == group)
    .map(|g| Json(g.clone()))
    .ok_or(not_found!())
}

#[get("/latest/<fetch>")]
pub async fn latest(
  fetch: FetchParam,
  db: &State<MongoPool>,
  cache: &State<Arc<Mutex<CachePool>>>,
) -> Result<Json<Snapshot>, ApiError> {
  let mut cache = cache.lock().await;
  if let Some(s) = cache.cached(fetch.clone().into()) {
    info!("Found cached {}!", s.uid);
    return Ok(Json(s));
  }

  info!("Trying to fetch {:?} snapshot from db", fetch);
  db.get_latest(fetch.clone().into())
    .await?
    .map(|s| {
      cache.try_cache_snapshot(&s);
      Json(s)
    })
    .ok_or(ApiError::SnapshotNotFound(fetch.to_string()))
}

#[get("/latest/<fetch>/<group>")]
pub async fn latest_group<'g>(
  fetch: FetchParam,
  group: &'g str,
  db: &State<MongoPool>,
  cache: &State<Arc<Mutex<CachePool>>>,
) -> Result<Json<TinySnapshot>, ApiError> {
  let mut cache = cache.lock().await;
  if let Some(s) = cache.cached(fetch.clone().into()) {
    info!("Found cached {}!", s.uid);
    return Ok(Json(s.tiny(group)));
  }

  info!("Trying to fetch {:?} snapshot from db", fetch);
  db.get_latest(fetch.clone().into())
    .await?
    .map(|s| {
      cache.try_cache_snapshot(&s);
      Json(s.tiny(group))
    })
    .ok_or(ApiError::SnapshotNotFound(fetch.to_string()))
}

#[get("/poll")]
pub async fn poll(cache: &State<Arc<Mutex<CachePool>>>) -> Result<Json<Poll>, ApiError> {
  Ok(Json(cache.lock().await.poll()))
}

#[get("/snapshot/<uid>")]
pub async fn snapshot_by_id<'a>(
  uid: &'a str,
  db: &State<MongoPool>,
  cache: &State<Arc<Mutex<CachePool>>>,
) -> Result<Json<Snapshot>, ApiError> {
  let mut cache = cache.lock().await;
  if let Some(s) = cache.cached_by_uid(uid) {
    info!("Found cached {}!", s.uid);
    return Ok(Json(s));
  }
  info!("Trying to fetch snapshot {} from db", uid);
  db.get_by_uid(uid)
    .await?
    .map(|s| {
      cache.try_cache_snapshot(&s);
      Json(s)
    })
    .ok_or(ApiError::SnapshotNotFound(uid.into()))
}

#[get("/cached")]
pub async fn cached(_secret: ApiKey, cache: &State<Arc<Mutex<CachePool>>>) -> Result<Json<Vec<Snapshot>>, ApiError> {
  let cache = cache.lock().await;
  Ok(Json(cache.collect_all().clone()))
}
