use maiq_parser::{fetch_n_parse, timetable::Snapshot};
use rocket::{http::Status, serde::json::Json, State};

use crate::{
  api::queries::FetchParam,
  db::{self, MongoClient},
};

use super::error::{ApiError, CustomApiError};

#[get("/")]
pub async fn index() -> Result<CustomApiError, ApiError> {
  Ok(CustomApiError { cause: "index_route", desc: "Hey there, stranger".into(), status: Status::Ok })
}

// todo: by group
#[get("/today")]
pub async fn today(mongo: &State<MongoClient>) -> Result<Json<Snapshot>, ApiError> {
  if let Some(x) = db::get_latest_today(&mongo).await? {
    info!("Returning cached snapshot");
    return Ok(Json(x));
  }

  Err(ApiError::ResourseNotFound("timetable for today".into()))
}

#[get("/next")]
pub async fn next(mongo: &State<MongoClient>) -> Result<Json<Snapshot>, ApiError> {
  if let Some(x) = db::get_latest_next(&mongo).await? {
    info!("Returning cached snapshot");
    return Ok(Json(x));
  }

  Err(ApiError::ResourseNotFound("timetable for next day".into()))
}

#[get("/<uid>")]
pub async fn snapshot_by_id<'a>(uid: &'a str, mongo: &State<MongoClient>) -> Result<Json<Snapshot>, ApiError> {
  if let Some(x) = db::get_by_uid(&mongo, uid).await? {
    return Ok(Json(x));
  }

  Err(ApiError::ResourseNotFound(format!("timetable #{}", uid)))
}

#[get("/naive/<mode>")]
pub async fn naive(mode: FetchParam) -> Result<Json<Snapshot>, ApiError> {
  let p = fetch_n_parse(&mode.into()).await?;
  Ok(Json(p.snapshot))
}
