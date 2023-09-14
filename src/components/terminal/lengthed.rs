use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use super::types::Terminal;

#[derive(Debug, Clone, Deserialize)]
pub struct LengthedTerminal {
    length: usize,
}

impl Terminal for LengthedTerminal {
    fn terminate(&mut self, _result: &str, token_count: usize) -> Result<bool> {
        Ok(token_count >= self.length)
    }

    fn clear(&mut self) {}

    fn clone(&self) -> Box<dyn Terminal> {
        Box::new(LengthedTerminal {
            length: self.length,
        })
    }
}

pub fn initialize_lenghted(_state: AppState, data: Option<Value>) -> Result<Box<dyn Terminal>> {
    Ok(Box::new(serde_json::from_value::<LengthedTerminal>(
        data.ok_or(Error::msg("Field must present to specify length!"))?,
    )?))
}
