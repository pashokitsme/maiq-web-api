use maiq_parser::error::ParserError;
use rocket::{
  http::{ContentType, Method, Status},
  response::{Responder, Result},
  serde::json::{self, Json},
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

#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum ApiError {
  #[error("Enviroment error")]
  Env(dotenvy::Error),

  #[error("{0}")]
  Json(json::Error<'static>),

  #[error("Failed to match ({1}) {0}. Try something else?")]
  NotFound(String, Method),

  // #[error("Method `{1}` on route `{0}` not allowed here")]
  // NotAllowed(String, Method),
  #[error("Database error: {0}")]
  Database(mongodb::error::Error),

  #[error("Requested resource `{0}` not found")]
  ResourseNotFound(String),

  #[error("{0}")]
  ParserError(ParserError),

  #[error("Unknown error")]
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
      ApiError::Env(..) => Status::InternalServerError,
      ApiError::Json(..) => Status::BadRequest,
      ApiError::NotFound { .. } => Status::NotFound,
      // ApiError::NotAllowed { .. } => Status::MethodNotAllowed,
      ApiError::Database(..) => Status::InternalServerError,
      ApiError::ResourseNotFound(..) => Status::NotFound,
      ApiError::ParserError(..) => Status::InternalServerError,
      ApiError::Unknown => Status::InternalServerError,
    }
  }

  fn cause(&self) -> &'static str {
    match self {
      ApiError::Env(..) => "env",
      ApiError::Json(..) => "json",
      ApiError::NotFound { .. } => "route_not_matched",
      // ApiError::NotAllowed { .. } => "method_not_allowed",
      ApiError::Database(..) => "db",
      ApiError::ResourseNotFound(..) => "resource_not_found",
      ApiError::ParserError(..) => "parser_error",
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

#[catch(404)]
pub fn not_found(req: &Request) -> ApiError {
  ApiError::NotFound(req.uri().path().to_string(), req.method())
}

// #[catch(500)]
// pub fn internal_server_error(req: &Request) -> () {}

// todo: make it work
/*
#[catch(405)]
pub fn method_not_allowed(req: &Request) -> ApiError {
  ApiError::NotAllowed(req.uri().path().to_string(), req.method())
}
*/
