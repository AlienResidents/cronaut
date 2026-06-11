//! Common error type that serializes cleanly to the frontend.

use serde::{Serialize, Serializer};

pub type Result<T> = std::result::Result<T, AppError>;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),

    #[error("command `{cmd}` failed (exit={code:?}): {stderr}")]
    Command {
        cmd: String,
        code: Option<i32>,
        stderr: String,
    },

    #[error("parse error: {0}")]
    Parse(String),

    #[error("invalid input: {0}")]
    Invalid(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("serde: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("tauri: {0}")]
    Tauri(#[from] tauri::Error),

    #[error("utf-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl AppError {
    pub fn invalid(msg: impl Into<String>) -> Self {
        Self::Invalid(msg.into())
    }
    pub fn parse(msg: impl Into<String>) -> Self {
        Self::Parse(msg.into())
    }
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}
