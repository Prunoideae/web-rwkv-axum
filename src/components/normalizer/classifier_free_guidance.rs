use anyhow::{Error, Result};
use ndarray::Array1;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use serde::Deserialize;
use serde_json::Value;

use crate::app::AppState;

use super::types::Normalizer;

const MAIN_STATE_INDEX: usize = 0; // assume the first state is the main state
const DYNAMIC_GAMMA_INDEX: usize = 1; // assume only the second state can be dynamic

#[derive(Debug, Deserialize, Clone)]
pub struct DynamicGamma {
    min: f32,
    max: f32,
    threshold: f32,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ClassifierFreeGuidanceData {
    static_gammas: Vec<f32>,
    #[serde(default)]
    dynamic_gamma: Option<DynamicGamma>,
}
#[derive(Debug, Clone)]
pub struct ClassifierFreeGuidance {
    cfg_data: ClassifierFreeGuidanceData,
    state: AppState,
}

impl ClassifierFreeGuidance {
    pub fn initialize(state: AppState, data: Option<Value>) -> Result<Box<dyn Normalizer>> {
        let data = serde_json::from_value::<ClassifierFreeGuidanceData>(
            data.ok_or(Error::msg("Field must present to specify static_gammas!"))?,
        )?;
        Ok(Box::new(ClassifierFreeGuidance {
            cfg_data: data,
            state,
        }))
    }
}

impl Normalizer for ClassifierFreeGuidance {
    fn normalize(&self, logits: Vec<Vec<f32>>) -> Vec<Vec<f32>> {
        fn calculate_entropy(probs: &Array1<f32>, log_probs: &Array1<f32>) -> f32 {
            -(probs * log_probs).sum()
        }

        let mut probs: Vec<Array1<f32>> = self
            .state
            .softmax_blocking(logits)
            .into_iter()
            .map(Array1::from_vec)
            .collect();
        let log_probs: Vec<Array1<f32>> = probs
            .iter()
            .map(|prob| Array1::from_vec(prob.par_iter().map(|x| x.ln()).collect::<Vec<f32>>()))
            .collect();
        let mut coefficients = vec![0.0; log_probs.len()];
        coefficients[MAIN_STATE_INDEX] = 1.0;
        if let Some(gamma) = &self.cfg_data.dynamic_gamma {
            let main_state_entropy: f32 =
                calculate_entropy(&probs[MAIN_STATE_INDEX], &log_probs[MAIN_STATE_INDEX]);
            let dynamic_state_entropy: f32 =
                calculate_entropy(&probs[DYNAMIC_GAMMA_INDEX], &log_probs[DYNAMIC_GAMMA_INDEX]);
            if main_state_entropy - dynamic_state_entropy > gamma.threshold {
                let max_entropy = (probs[MAIN_STATE_INDEX].len() as f32).ln();
                let gamma_value = gamma.min
                    + (gamma.max - gamma.min) * (1.0 - dynamic_state_entropy / max_entropy);
                coefficients[MAIN_STATE_INDEX] = 1.0 - gamma_value;
                coefficients[DYNAMIC_GAMMA_INDEX] = gamma_value;
            }
        }
        {
            let mut j = 0;
            for gamma in self.cfg_data.static_gammas.iter() {
                if j == MAIN_STATE_INDEX {
                    j += 1;
                }
                if j == DYNAMIC_GAMMA_INDEX && self.cfg_data.dynamic_gamma.is_some() {
                    j += 1;
                }
                coefficients[j] = *gamma;
                coefficients[MAIN_STATE_INDEX] -= gamma;
                j += 1;
            }
        }
        let mut main_probs: Array1<f32> = coefficients
            .iter()
            .zip(log_probs)
            .map(|(x, y)| *x * &y)
            .reduce(|x, y| x + y)
            .unwrap();
        main_probs.par_mapv_inplace(|x| x.exp());
        probs[MAIN_STATE_INDEX] = main_probs;
        probs.iter().map(|x| x.to_vec()).collect()
    }

    fn clone(&self) -> Box<dyn Normalizer> {
        Box::new(ClassifierFreeGuidance {
            cfg_data: self.cfg_data.clone(),
            state: self.state.clone(),
        })
    }
}
