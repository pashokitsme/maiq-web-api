use std::sync::Arc;

use maiq_parser::{default::DefaultGroup, Fetch, Snapshot, TinySnapshot};
use rocket::{http::Status, serde::json::Json, State};
use tokio::sync::Mutex;

use crate::{
  api::FetchParam,
  storage::{
    cache::{CachePool, Poll},
    mongo::MongoPool,
    SnapshotPool,
  },
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
  let fetch: Fetch = fetch.into();
  if let Ok(Some(s)) = cache.latest(fetch.clone()).await {
    info!("Found cached {}!", s.uid);
    return Ok(Json(s));
  }

  info!("Trying to fetch {:?} snapshot from db", fetch);
  match db.latest(fetch.clone()).await? {
    Some(s) => {
      cache.save(&s).await?;
      Ok(Json(s))
    }
    None => Err(ApiError::SnapshotNotFound(format!("{:?}", fetch))),
  }
}

#[get("/latest/<fetch>/<group>")]
pub async fn latest_group<'g>(
  fetch: FetchParam,
  group: &'g str,
  db: &State<MongoPool>,
  cache: &State<Arc<Mutex<CachePool>>>,
) -> Result<Json<TinySnapshot>, ApiError> {
  let mut cache = cache.lock().await;
  let fetch: Fetch = fetch.into();
  if let Ok(Some(s)) = cache.latest(fetch.clone()).await {
    info!("Found cached {}!", s.uid);
    return Ok(Json(s.tiny(group)));
  }

  info!("Trying to fetch {:?} snapshot from db", fetch);
  match db.latest(fetch.clone()).await? {
    Some(s) => {
      cache.save(&s).await?;
      Ok(Json(s.tiny(group)))
    }
    None => Err(ApiError::SnapshotNotFound(format!("{:?}", fetch))),
  }
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
  if let Ok(Some(s)) = cache.by_uid(uid).await {
    info!("Found cached {}!", s.uid);
    return Ok(Json(s));
  }
  info!("Trying to fetch snapshot {} from db", uid);
  match db.by_uid(uid).await? {
    Some(s) => {
      cache.save(&s).await?;
      Ok(Json(s))
    }
    None => Err(ApiError::SnapshotNotFound(uid.into())),
  }
}

#[get("/cached")]
pub async fn cached(_secret: ApiKey, cache: &State<Arc<Mutex<CachePool>>>) -> Result<Json<Vec<Snapshot>>, ApiError> {
  let cache = cache.lock().await;
  Ok(Json(cache.collect_all().clone()))
}
