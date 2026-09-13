use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;
use crate::helpers::suid::suid;

/// Chuẩn hóa toàn bộ lỗi của hệ thống thành một Enum duy nhất
#[allow(dead_code)]
#[derive(Debug)]
pub enum AppError {
    IoError(std::io::Error),
    TtsEngineFailed(String),
    Custom(StatusCode, String),
}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::IoError(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Lỗi I/O đĩa cứng"),
            AppError::TtsEngineFailed(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg.as_str()),
            AppError::Custom(code, msg) => (*code, msg.as_str()),
        };

        let req_id = suid();

        error!(
            category = "Error",
            req_id = %req_id,
            error_detail = ?self,
            "API Error"
        );

        let body = Json(json!({
            "code": status.as_u16(),
            "message": message,
            "req_id": req_id
        }));

        (status, body).into_response()
    }
}
