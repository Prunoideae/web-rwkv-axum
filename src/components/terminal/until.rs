use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use super::types::Terminal;

#[derive(Debug, Clone, Deserialize)]
pub struct UntilTerminal {
    until: String,
}

impl Terminal for UntilTerminal {
    fn terminate(&mut self, result: &str, _token_count: usize) -> Result<bool> {
        Ok(result.contains(&self.until))
    }

    fn clone(&self) -> Box<dyn Terminal> {
        Box::new(UntilTerminal {
            until: self.until.clone(),
        })
    }
}

pub fn intialize_until(_state: AppState, data: Option<Value>) -> Result<Box<dyn Terminal>> {
    Ok(Box::new(serde_json::from_value::<UntilTerminal>(
        data.ok_or(Error::msg("Field must present to specify data!"))?,
    )?))
}
