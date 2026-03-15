use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Custom error type for server errors
#[derive(Debug, Serialize, Deserialize)]
pub struct ServerError {
    pub message: String,
}

impl ServerError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ServerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Server error: {}", self.message)
    }
}

impl ResponseError for ServerError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .json(serde_json::json!({
                "error": self.message,
                "status": self.status_code().as_u16()
            }))
    }
}

impl From<webots_sys::WebotsError> for ServerError {
    fn from(err: webots_sys::WebotsError) -> Self {
        Self::new(&format!("Webots error: {}", err))
    }
}

impl From<std::io::Error> for ServerError {
    fn from(err: std::io::Error) -> Self {
        Self::new(&format!("IO error: {}", err))
    }
}

impl From<actix_multipart::MultipartError> for ServerError {
    fn from(err: actix_multipart::MultipartError) -> Self {
        Self::new(&format!("Multipart error: {}", err))
    }
}

impl From<tempfile::PathPersistError> for ServerError {
    fn from(err: tempfile::PathPersistError) -> Self {
        Self::new(&format!("Temp file error: {}", err))
    }
}