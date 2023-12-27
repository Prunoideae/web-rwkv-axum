use anyhow::{Error, Result};

use rayon::iter::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, components::InferenceInterruption};

use super::types::{penalty_transform, PenaltyMode, Transformer};

#[derive(Debug, Deserialize, Clone, Copy)]
struct PenaltyData {
    alpha_occurrence: f32,
    alpha_presence: f32,
    #[serde(default)]
    mode: PenaltyMode,
}

#[derive(Debug)]
pub struct GlobalPenalty {
    data: PenaltyData,
    record: Vec<f32>,
    presence: Vec<f32>,
}

impl Transformer for GlobalPenalty {
    fn update(&mut self, prompt: &Vec<u16>) -> Result<(), InferenceInterruption> {
        for &token in prompt {
            match self.data.mode {
                PenaltyMode::Subtract => self.record[token as usize] += self.data.alpha_occurrence,
                PenaltyMode::Divide => self.record[token as usize] *= self.data.alpha_occurrence,
            }

            self.presence[token as usize] = self.data.alpha_presence;
        }
        Ok(())
    }

    fn transform(&self, logits: Vec<f32>) -> Vec<f32> {
        penalty_transform(self.data.mode, logits, &self.record, &self.presence)
    }

    fn clear(&mut self) {
        self.record = match self.data.mode {
            PenaltyMode::Subtract => vec![0.0; 65536],
            PenaltyMode::Divide => vec![1.0; 65536],
        };
        self.presence = vec![0.0; 65536];
    }

    fn clone(&self) -> Box<dyn Transformer> {
        Box::new(GlobalPenalty {
            data: self.data,
            presence: self.presence.clone(),
            record: self.record.clone(),
        })
    }
}

pub fn initialize_global(_state: AppState, data: Option<Value>) -> Result<Box<dyn Transformer>> {
    let data: PenaltyData = serde_json::from_value(data.ok_or(Error::msg(
        "Field must present to specify alpha presence and occurrence!",
    ))?)?;
    Ok(Box::new(GlobalPenalty {
        data,
        presence: vec![0.0; 65536],
        record: match data.mode {
            PenaltyMode::Subtract => vec![0.0; 65536],
            PenaltyMode::Divide => vec![1.0; 65536],
        },
    }))
}
