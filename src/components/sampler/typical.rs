use anyhow::{Error, Result};
use ndarray::{self, s, Array};
use rand::distributions::{Distribution, WeightedIndex};
use serde::Deserialize;
use serde_json::Value;

use crate::{app::AppState, components::sampler::utils::argsort};

use super::{types::Sampler, utils};
#[derive(Debug, Deserialize)]
struct TypicalSamplerData {
    tau: f32,
    temp: f32,
}

#[derive(Debug)]
pub struct TypicalSampler {
    tau: f32,
    temp: f32,
    vocab_size: usize,
}

impl TypicalSampler {
    pub fn initialize(state: AppState, data: Option<Value>) -> Result<Box<dyn Sampler>> {
        let data = serde_json::from_value::<TypicalSamplerData>(data.ok_or(Error::msg(
            "Invalid BNFData. Example format:{
                tau: f32,
                temp: f32,
            }",
        ))?)?;
        Ok(Box::new(TypicalSampler {
            tau: data.tau,
            temp: data.temp,
            vocab_size: state.0.tokenizer.bytes_to_token_index().len(),
        }))
    }
}

impl Sampler for TypicalSampler {
    fn update(
        &mut self,
        _tokens: &Vec<Vec<u16>>,
    ) -> anyhow::Result<(), crate::components::InferenceInterruption> {
        Ok(())
    }

    fn sample(&self, probs: Vec<Vec<f32>>) -> u16 {
        assert!(probs.len() == 1);
        let mut probs = probs;
        let mut probs = probs.pop().unwrap();
        probs.truncate(self.vocab_size);
        let mut probs = Array::from_vec(probs);
        let mut logits = probs.clone();
        logits.par_mapv_inplace(|x| -x.ln());
        let entropy = (&probs * &logits).fold(0.0, |x, y| if y.is_nan() { x } else { x + y });
        logits -= entropy;
        logits.par_mapv_inplace(|x| if x.is_nan() { f32::MAX } else { x.abs() });
        let sorted_ids = argsort(logits.view());
        utils::sort_by_indices(logits.view_mut(), sorted_ids.view());
        utils::sort_by_indices(probs.view_mut(), sorted_ids.view());
        let mut temp = 0.0;
        let cut_off = probs
            .iter()
            .position(|x| {
                temp += x;
                temp >= self.tau
            })
            .unwrap_or(probs.len() - 1);
        let mut probs = probs.slice_move(s![..cut_off + 1]);
        if self.temp != 1.0 {
            probs.par_mapv_inplace(|x| x.powf(1.0 / self.temp));
        }
        let mut rng = rand::thread_rng();
        // println!("{:?}",probs);
        let token_id = if probs.len() == 1 {
            sorted_ids[0] as u16
        } else {
            sorted_ids[WeightedIndex::new(probs.as_slice().unwrap())
                .unwrap()
                .sample(&mut rng)] as u16
        };
        token_id
    }

    fn clear(&mut self) {}

    fn clone(&self) -> Box<dyn Sampler> {
        Box::new(TypicalSampler {
            tau: self.tau,
            temp: self.temp,
            vocab_size: self.vocab_size,
        })
    }
}
