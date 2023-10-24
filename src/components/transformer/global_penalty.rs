use anyhow::{Error, Result};
use ndarray::Array1;
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, components::InferenceInterruption};

use super::types::Transformer;

#[derive(Debug, Deserialize, Clone)]
struct PenaltyData {
    alpha_occurrence: f32,
    alpha_presence: f32,
}

#[derive(Debug)]
pub struct GlobalPenalty {
    data: PenaltyData,
    record: Array1<f32>,
    presence: Array1<f32>,
}

impl Transformer for GlobalPenalty {
    fn update(&mut self, prompt: &Vec<u16>) -> Result<(), InferenceInterruption> {
        for &token in prompt {
            self.record[token as usize] += self.data.alpha_occurrence;
            self.presence[token as usize] = self.data.alpha_presence;
        }
        Ok(())
    }

    fn transform(&self, logits: Vec<f32>) -> Vec<f32> {
        (Array1::from_vec(logits) - &self.record - &self.presence).into_raw_vec()
    }

    fn clear(&mut self) {
        self.record = Array1::zeros(65536);
        self.presence = Array1::zeros(65536);
    }

    fn clone(&self) -> Box<dyn Transformer> {
        Box::new(GlobalPenalty {
            data: self.data.clone(),
            presence: self.presence.clone(),
            record: self.record.clone(),
        })
    }
}

pub fn initialize_global(_state: AppState, data: Option<Value>) -> Result<Box<dyn Transformer>> {
    Ok(Box::new(GlobalPenalty {
        data: serde_json::from_value(data.ok_or(Error::msg(
            "Field must present to specify alpha presence and occurrence!",
        ))?)?,
        presence: Array1::zeros(65536),
        record: Array1::zeros(65536),
    }))
}
