use maiq_parser::error::ParserError;
use rocket::{
  http::{ContentType, Method, Status},
  response::{Responder, Result},
  serde::json::Json,
  Request, Response,
};
use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Serialize)]
pub struct CustomApiError {
  pub cause: &'static str,
  #[serde(skip_serializing)]
  pub status: Status,
  pub desc: String,
}

#[derive(Error, Debug)]
pub enum ApiError {
  #[error("Failed to match ({1}) {0}. Try something else?")]
  NotFound(String, Method),

  #[error("Database error: {0}")]
  Database(mongodb::error::Error),

  #[error("Requested snapshot `{0}` not found")]
  SnapshotNotFound(String),

  #[error("Requested default for `{1}` for `{0}` not found")]
  DefaultNotFound(String, String),

  #[error("{0}")]
  ParserError(ParserError),

  #[error("Invalid API Key")]
  InvalidApiKey,

  #[error("Internal server error")]
  Unknown,
}

impl From<mongodb::error::Error> for ApiError {
  fn from(err: mongodb::error::Error) -> Self {
    ApiError::Database(err)
  }
}

impl From<ParserError> for ApiError {
  fn from(err: ParserError) -> Self {
    ApiError::ParserError(err)
  }
}

impl Into<CustomApiError> for ApiError {
  fn into(self) -> CustomApiError {
    CustomApiError { cause: self.cause(), desc: self.to_string(), status: self.status_code() }
  }
}

impl ApiError {
  fn status_code(&self) -> Status {
    match self {
      ApiError::NotFound { .. } => Status::NotFound,
      ApiError::Database(..) => Status::InternalServerError,
      ApiError::SnapshotNotFound(..) => Status::NotFound,
      ApiError::DefaultNotFound(..) => Status::NotFound,
      ApiError::ParserError(..) => Status::InternalServerError,
      ApiError::InvalidApiKey => Status::Unauthorized,
      ApiError::Unknown => Status::InternalServerError,
    }
  }

  fn cause(&self) -> &'static str {
    match self {
      ApiError::NotFound { .. } => "route_not_matched",
      ApiError::Database(..) => "db_err",
      ApiError::SnapshotNotFound(..) => "snapshot_not_found",
      ApiError::DefaultNotFound(..) => "default_not_found",
      ApiError::ParserError(..) => "internal_parser_err",
      ApiError::InvalidApiKey => "invalid_api_key",
      ApiError::Unknown => "unknown",
    }
  }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for ApiError {
  fn respond_to(self, request: &Request) -> Result<'o> {
    let err: CustomApiError = self.into();
    err.respond_to(request)
  }
}

impl<'r, 'o: 'r> Responder<'r, 'o> for CustomApiError {
  fn respond_to(self, request: &Request) -> Result<'o> {
    let res = Json(&self).respond_to(request)?;
    Ok(
      Response::build_from(res)
        .status(self.status)
        .header(ContentType::JSON)
        .finalize(),
    )
  }
}

#[catch(401)]
pub fn unauthorized(_: &Request) -> ApiError {
  ApiError::InvalidApiKey
}

#[catch(404)]
pub fn not_found(req: &Request) -> ApiError {
  ApiError::NotFound(req.uri().path().to_string(), req.method())
}

#[catch(500)]
pub fn internal_server_error(_: &Request) -> ApiError {
  ApiError::Unknown
}

// todo: make it work
/*
#[catch(405)]
pub fn method_not_allowed(req: &Request) -> ApiError {
  ApiError::NotAllowed(req.uri().path().to_string(), req.method())
}
*/
