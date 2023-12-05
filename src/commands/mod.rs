use anyhow::{Error, Ok, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, register_handlers};

mod handle_infer;
mod handle_states;
mod handle_pipeline;

pub mod types;

#[derive(Debug, Deserialize)]
pub struct TextCommand {
    pub echo_id: String,
    command: String,
    data: Option<Value>,
}

impl TextCommand {
    pub async fn handle(&self, state: AppState) -> Result<Value> {
        register_handlers!(
            self,
            state,
            [
                // States
                handle_states::create_state,
                handle_states::copy_state,
                handle_states::update_state,
                handle_states::delete_state,
                handle_states::dump_state,
                //Infer
                handle_infer::infer,
                //Pipeline
                handle_pipeline::create_pipeline,
                handle_pipeline::copy_pipeline,
                handle_pipeline::delete_pipeline,
                handle_pipeline::reset_pipeline,
                handle_pipeline::modify_pipeline,
            ]
        )
    }
}
