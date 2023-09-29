use std::collections::VecDeque;

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
    window_size: usize,
}

#[derive(Debug)]
pub struct SlidingPenalty {
    data: PenaltyData,
    record: Array1<f32>,
    presence: Array1<f32>,
    history: VecDeque<u16>,
}

impl Transformer for SlidingPenalty {
    fn update(&mut self, prompt: &Vec<u16>) -> Result<(), InferenceInterruption> {
        for &token in prompt {
            if self.history.len() == self.data.window_size {
                // Remove the most ancient token
                let removed = self.history.pop_front().unwrap();
                self.record[removed as usize] -= self.data.alpha_occurrence;
                if self.record[removed as usize] == 0f32 {
                    self.presence[removed as usize] = 0f32;
                }
            }
            self.history.push_back(token);
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
        Box::new(SlidingPenalty {
            data: self.data.clone(),
            presence: self.presence.clone(),
            record: self.record.clone(),
            history: self.history.clone(),
        })
    }
}

pub fn initialize_sliding(_state: AppState, data: Option<Value>) -> Result<Box<dyn Transformer>> {
    let data = serde_json::from_value::<PenaltyData>(data.ok_or(Error::msg(
        "Field must present to specify alpha presence, occurrence and history size!",
    ))?)?;
    let window_size = data.window_size;
    Ok(Box::new(SlidingPenalty {
        data,
        record: Array1::zeros(65536),
        presence: Array1::zeros(65536),
        history: VecDeque::with_capacity(window_size),
    }))
}
