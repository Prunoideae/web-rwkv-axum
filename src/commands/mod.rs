use anyhow::{Error, Ok, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::SharedState;

mod handle_states;
pub mod types;

#[derive(Debug, Deserialize)]
pub struct TextCommand {
    pub echo_id: String,
    command: String,
    data: Value,
}

impl TextCommand {
    pub async fn handle(&self, state: SharedState) -> Result<Value> {
        match self.command.as_str() {
            "create_state" => handle_states::create_state(self.data.clone(), state).await,
            "copy_state" => handle_states::copy_state(self.data.clone(), state).await,
            "delete_state" => handle_states::delete_state(self.data.clone(), state).await,
            "echo" => Ok(self.data.clone()),
            _ => Err(Error::msg("Unknown command!")),
        }
    }
}
