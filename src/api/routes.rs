use rocket::serde::json::{serde_json::json, Value};

use super::error::ApiError;

#[get("/")]
pub async fn index() -> Result<Value, ApiError> {
  Ok(json!({ "ok": true }))
}
