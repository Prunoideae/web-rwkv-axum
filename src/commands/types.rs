use anyhow::Error;
use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Serialize)]
pub struct CommandError {
    echo_id: String,
    status: &'static str,
    error: String,
}

impl CommandError {
    pub fn new(id: String, error: Error) -> Self {
        Self {
            echo_id: id,
            status: "error",
            error: error.to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CommandSuccess {
    echo_id: String,
    status: &'static str,
    result: Value,
}

impl CommandSuccess {
    pub fn new(id: String, result: Value) -> Self {
        Self {
            echo_id: id,
            status: "success",
            result,
        }
    }
}
