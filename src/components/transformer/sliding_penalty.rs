use std::collections::VecDeque;

use anyhow::{Error, Result};
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, components::InferenceInterruption};

use super::types::{penalty_transform, PenaltyMode, Transformer};

#[derive(Debug, Deserialize, Clone, Copy)]
struct PenaltyData {
    alpha_occurrence: f32,
    alpha_presence: f32,
    window_size: usize,
    #[serde(default)]
    mode: PenaltyMode,
}

#[derive(Debug)]
pub struct SlidingPenalty {
    data: PenaltyData,
    record: Vec<f32>,
    presence: Vec<f32>,
    history: VecDeque<u16>,
}

impl Transformer for SlidingPenalty {
    fn update(&mut self, prompt: &Vec<u16>) -> Result<(), InferenceInterruption> {
        const TOLERANCE: f32 = 0.001;
        for &token in prompt {
            if self.history.len() == self.data.window_size {
                // Remove the most ancient token
                let removed = self.history.pop_front().unwrap();
                match self.data.mode {
                    PenaltyMode::Subtract => {
                        self.record[removed as usize] -= self.data.alpha_occurrence;
                        if self.record[removed as usize].abs() < TOLERANCE {
                            self.presence[removed as usize] = 0f32;
                        }
                    }
                    PenaltyMode::Divide => {
                        self.record[removed as usize] /= self.data.alpha_occurrence;
                        if (self.record[removed as usize] - 1.0).abs() < TOLERANCE {
                            self.presence[removed as usize] = 1f32;
                        }
                    }
                }
            }
            self.history.push_back(token);
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
        self.presence = match self.data.mode {
            PenaltyMode::Subtract => vec![0.0; 65536],
            PenaltyMode::Divide => vec![1.0; 65536],
        };
    }

    fn clone(&self) -> Box<dyn Transformer> {
        Box::new(SlidingPenalty {
            data: self.data,
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
    if PenaltyMode::Divide == data.mode{
        if data.alpha_presence==0.0
        {
            return Err(Error::msg("alpha presence in divide mode cannot be zero!"));
        }
        if data.alpha_occurrence == 0.0
        {
            return Err(Error::msg("alpha presence in divide mode cannot be zero!"));
        }
    }
    let window_size = data.window_size;
    Ok(Box::new(SlidingPenalty {
        data,
        presence: match data.mode {
            PenaltyMode::Subtract => vec![0.0; 65536],
            PenaltyMode::Divide => vec![1.0; 65536],
        },
        record: match data.mode {
            PenaltyMode::Subtract => vec![0.0; 65536],
            PenaltyMode::Divide => vec![1.0; 65536],
        },
        history: VecDeque::with_capacity(window_size),
    }))
}
