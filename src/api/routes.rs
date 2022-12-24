use std::sync::Arc;

use maiq_parser::{fetch_n_parse, timetable::Snapshot};
use rocket::{http::Status, serde::json::Json, State};
use tokio::sync::Mutex;

use crate::{
  api::queries::FetchParam,
  cache::Cache,
  db::{self, MongoPool},
};

use super::error::{ApiError, CustomApiError};

#[get("/")]
pub async fn index() -> Result<CustomApiError, ApiError> {
  Ok(CustomApiError { cause: "index_route", desc: "Hey there, stranger".into(), status: Status::Ok })
}

// todo: grouping by group name
#[get("/latest/<fetch>")]
pub async fn latest(fetch: FetchParam, mongo: &State<MongoPool>) -> Result<Json<Snapshot>, ApiError> {
  let snapshot = match fetch {
    FetchParam::Today => db::get_latest_today(&mongo).await?,
    FetchParam::Tomorrow => db::get_latest_next(&mongo).await?,
  };
  if let Some(x) = snapshot {
    debug!("Returning cached snapshot #{}", x.uid);
    return Ok(Json(x));
  }

  Err(ApiError::NoTimetable())
}

#[get("/poll")]
pub async fn poll(cache: &State<Arc<Mutex<Cache>>>) -> Result<Json<Cache>, ApiError> {
  Ok(Json(cache.lock().await.clone()))
}

#[get("/snapshot/<uid>")]
pub async fn snapshot_by_id<'a>(uid: &'a str, mongo: &State<MongoPool>) -> Result<Json<Snapshot>, ApiError> {
  if let Some(x) = db::get_by_uid(&mongo, uid).await? {
    return Ok(Json(x));
  }

  Err(ApiError::ResourseNotFound(format!("timetable #{}", uid)))
}

#[get("/naive/<fetch>")]
pub async fn naive(fetch: FetchParam) -> Result<Json<Snapshot>, ApiError> {
  let p = fetch_n_parse(&fetch.into()).await?;
  Ok(Json(p.snapshot))
}
