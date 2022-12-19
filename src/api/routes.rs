use maiq_parser::{fetch_n_parse, timetable::Day, Fetch};
use rocket::serde::json::{serde_json::json, Json, Value};

use super::error::ApiError;

#[get("/")]
pub async fn index() -> Result<Value, ApiError> {
  Ok(json!({ "ok": true }))
}

#[get("/naive/<mode>")]
pub async fn get_instantly(mode: &str) -> Result<Json<Day>, ApiError> {
  let mode = match mode {
    "today" => Fetch::Today,
    "tomorrow" => Fetch::Tomorrow,
    _ => return Err(ApiError::ResourseNotFound(mode.to_string())),
  };
  let day = fetch_n_parse(mode).await?;
  Ok(Json(day.day))
}
