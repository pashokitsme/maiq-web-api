use maiq_api_wrapper::Poll;
use maiq_parser::{default::DefaultGroup, Snapshot, TinySnapshot};
use rocket::{http::Status, serde::json::Json};

use crate::{
  api::{map_weekday, CachePool, FetchParam, MongoPool},
  storage::SnapshotPool,
};

use super::{
  error::{ApiError, CustomApiError},
  ApiKey, DateParam,
};

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
pub async fn latest(fetch: FetchParam, db: &MongoPool, cache: &CachePool) -> Result<Json<Snapshot>, ApiError> {
  if let Ok(Some(s)) = cache.read().await.latest(*fetch).await {
    return Ok(Json(s));
  }

  info!("Trying to fetch {:?} snapshot from db", *fetch);
  match db.latest(*fetch).await? {
    Some(s) => {
      cache.write().await.save(&s).await?;
      Ok(Json(s))
    }
    None => Err(ApiError::SnapshotNotFound(format!("{:?}", fetch))),
  }
}

#[get("/latest/<fetch>/<group>")]
pub async fn latest_group(
  fetch: FetchParam,
  group: &str,
  db: &MongoPool,
  cache: &CachePool,
) -> Result<Json<TinySnapshot>, ApiError> {
  if let Ok(Some(s)) = cache.read().await.latest(*fetch).await {
    return Ok(Json(s.tiny(group)));
  }

  info!("Trying to fetch {:?} snapshot from db", fetch);
  match db.latest(*fetch).await? {
    Some(s) => {
      cache.write().await.save(&s).await?;
      Ok(Json(s.tiny(group)))
    }
    None => Err(ApiError::SnapshotNotFound(format!("{:?}", fetch))),
  }
}

#[get("/poll")]
pub async fn poll(cache: &CachePool) -> Result<Json<Poll>, ApiError> {
  Ok(Json(cache.read().await.poll()))
}

#[get("/date/<date>")]
pub async fn snapshot_by_date(date: Result<DateParam, ApiError>, db: &MongoPool) -> Result<Json<Snapshot>, ApiError> {
  let date = date?.0;
  db.by_date(date)
    .await?
    .map(Json)
    .ok_or_else(|| ApiError::SnapshotNotFound(format!("{}", date)))
}

#[get("/uid/<uid>")]
pub async fn snapshot_by_id(uid: &str, db: &MongoPool, cache: &CachePool) -> Result<Json<Snapshot>, ApiError> {
  if let Ok(Some(s)) = cache.read().await.by_uid(uid).await {
    return Ok(Json(s));
  }
  info!("Trying to fetch snapshot {} from db", uid);
  match db.by_uid(uid).await? {
    Some(s) => {
      cache.write().await.save(&s).await?;
      Ok(Json(s))
    }
    None => Err(ApiError::SnapshotNotFound(uid.to_string())),
  }
}

#[get("/cached")]
pub async fn cached(_secret: ApiKey, cache: &CachePool) -> Result<Json<Vec<Snapshot>>, ApiError> {
  Ok(Json(cache.read().await.collect_all()))
}
