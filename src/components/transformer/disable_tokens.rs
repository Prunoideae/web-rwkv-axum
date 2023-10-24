use anyhow::{Error, Result};
use ndarray::Array1;
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use super::types::Transformer;

const DISABLED: f32 = -1e30;

#[derive(Debug, Deserialize)]
pub struct DisableTokensData {
    tokens: Vec<u16>,
}

#[derive(Debug)]
pub struct DisableTokens {
    tokens: Array1<f32>,
}

impl Transformer for DisableTokens {
    fn transform(&self, logits: Vec<f32>) -> Vec<f32> {
        (Array1::from_vec(logits) + &self.tokens).into_raw_vec()
    }

    fn clone(&self) -> Box<dyn Transformer> {
        Box::new(DisableTokens {
            tokens: self.tokens.clone(),
        })
    }
}

pub fn initialize_disable(_state: AppState, data: Option<Value>) -> Result<Box<dyn Transformer>> {
    let DisableTokensData { tokens } = data
        .ok_or(Error::msg("Field must present to specify tokens to ban!"))
        .map(|data| serde_json::from_value(data))??;
    let mut tokens_offset = Array1::zeros(65536);
    for token in tokens {
        tokens_offset[token as usize] = DISABLED
    }
    Ok(Box::new(DisableTokens {
        tokens: tokens_offset,
    }))
}
