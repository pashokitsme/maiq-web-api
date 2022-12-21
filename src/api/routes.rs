use maiq_parser::{fetch_n_parse, timetable::Snapshot, Fetch};
use rocket::{http::Status, serde::json::Json, State};

use crate::{
  api::queries::FetchParam,
  db::{self, MongoClient},
};

use super::error::{ApiError, CustomApiError};

#[get("/")]
pub async fn index() -> Result<CustomApiError, ApiError> {
  Ok(CustomApiError { cause: "index", desc: "Hey there, stranger".into(), status: Status::Ok })
}

// todo: by group
#[get("/today")]
pub async fn today(mongo: &State<MongoClient>) -> Result<Json<Snapshot>, ApiError> {
  if let Some(x) = db::get_latest_today(&mongo).await? {
    info!("Returning cached snapshot");
    return Ok(Json(x));
  }

  info!("Parsing new snapshot");
  let snapshot = fetch_n_parse(Fetch::Today).await?.snapshot;
  db::save(&mongo, &snapshot).await?;
  Ok(Json(snapshot))
}

#[get("/next")]
pub async fn next(mongo: &State<MongoClient>) -> Result<Json<Snapshot>, ApiError> {
  if let Some(x) = db::get_latest_next(&mongo).await? {
    info!("Returning cached snapshot");
    return Ok(Json(x));
  }

  info!("Parsing new snapshot");
  let snapshot = fetch_n_parse(Fetch::Tomorrow).await?.snapshot;
  db::save(&mongo, &snapshot).await?;
  Ok(Json(snapshot))
}

#[get("/naive/<mode>")]
pub async fn naive(mode: FetchParam) -> Result<Json<Snapshot>, ApiError> {
  let p = fetch_n_parse(mode.into()).await?;
  Ok(Json(p.snapshot))
}

#[get("/update/<mode>")]
pub async fn update(mode: FetchParam, mongo: &State<MongoClient>) -> Result<(), ApiError> {
  let parsed = fetch_n_parse(mode.into()).await?;
  db::save(&*mongo, &parsed.snapshot).await?;
  Ok(())
}
