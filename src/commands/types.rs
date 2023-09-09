use anyhow::Error;
use serde::Serialize;
use serde_json::Value;
use tokio::time::Instant;

#[derive(Debug, Serialize)]
pub struct CommandError {
    echo_id: Option<String>,
    status: &'static str,
    error: String,
}

impl CommandError {
    pub fn new(id: String, error: Error) -> Self {
        Self {
            echo_id: Some(id),
            status: "error",
            error: error.to_string(),
        }
    }

    pub fn new_raw(error: Error) -> Self {
        Self {
            echo_id: None,
            status: "error",
            error: format!("{}", error.to_string()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CommandSuccess {
    echo_id: String,
    status: &'static str,
    result: Value,
    duration_ms: usize,
}

impl CommandSuccess {
    pub fn new(id: String, result: Value, duration: Instant) -> Self {
        Self {
            echo_id: id,
            status: "success",
            result,
            duration_ms: duration.elapsed().as_millis() as usize,
        }
    }
}
