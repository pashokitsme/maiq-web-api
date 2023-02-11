use maiq_api_models::polling::Poll;
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
pub async fn index() -> Result<CustomApiError, ApiError> {
  Ok(CustomApiError { cause: "index_route", desc: "Hey there, stranger".into(), status: Status::Ok })
}

#[get("/default/<weekday>/<group>")]
pub fn default(weekday: &str, group: &str) -> Result<Json<DefaultGroup>, ApiError> {
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
  Ok(cache.read().await.poll())
}

#[get("/date/<date>")]
pub async fn snapshot_by_date(date: Result<DateParam, ApiError>, db: &MongoPool) -> Result<Json<Snapshot>, ApiError> {
  let date = date?.0;
  let s = db
    .by_date(date)
    .await?
    .ok_or_else(|| ApiError::SnapshotNotFound(format!("{}", date)))?;

  Ok(Json(s))
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
  Ok(Json(cache.read().await.collect_all().clone()))
}
