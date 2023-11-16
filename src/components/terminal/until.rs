use std::collections::HashSet;

use anyhow::{Error, Result};
use nohash_hasher::BuildNoHashHasher;
use rayon::prelude::*;
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use super::types::Terminal;

#[derive(Debug, Clone)]
pub struct UntilTerminal {
    state: AppState,
    until: String,
    cap: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
struct UntilData {
    until: String,
    cap: Option<usize>,
}

impl Terminal for UntilTerminal {
    fn terminate(&mut self, result: &Vec<u16>, token_count: usize) -> Result<bool> {
        Ok(self.cap.unwrap_or(usize::MAX) <= token_count
            && String::from_utf8_lossy(&self.state.0.tokenizer.decode(&result)?)
                .contains(&self.until))
    }

    fn clone(&self) -> Box<dyn Terminal> {
        Box::new(UntilTerminal {
            state: self.state.clone(),
            until: self.until.clone(),
            cap: self.cap.clone(),
        })
    }
}

pub fn intialize_until(state: AppState, data: Option<Value>) -> Result<Box<dyn Terminal>> {
    let UntilData { until, cap } = serde_json::from_value::<UntilData>(
        data.ok_or(Error::msg("Field must present to specify data!"))?,
    )?;

    Ok(Box::new(UntilTerminal {
        state: state.clone(),
        until,
        cap,
    }))
}
