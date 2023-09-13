use crate::states::InferenceInterruption;
use bit_set::BitSet;
use bnf_sampler::sampler::{AcceptTokenResult, PossibleTokensResult, Sampler};

use super::types::Transformer;
#[derive(Debug, Clone)]
pub struct BNFConstraint {
    sampler: Sampler,
    current_token_ids: BitSet,
}

impl Transformer for BNFConstraint {
    fn update(&mut self, prompt: &Vec<u16>) -> Result<(), InferenceInterruption> {
        for token_id in prompt {
            match self.sampler.accept_a_token(Some(*token_id as u32)) {
                AcceptTokenResult::End => return Result::Err(InferenceInterruption::Exhaustion),
                AcceptTokenResult::Failed => {
                    return Result::Err(InferenceInterruption::Error(anyhow::anyhow!(
                        "Token {token_id} is rejected by BNF schema."
                    )))
                }
                AcceptTokenResult::Continue => {}
            }
        }
        self.current_token_ids = match self.sampler.all_possible_next_tokens(None) {
            PossibleTokensResult::Continue(token_ids) => token_ids.clone(),
            _ => unreachable!(),
        };
        Ok(())
    }

    fn transform(&self, logits: Vec<f32>) -> Vec<f32> {
        let mut logits = logits;
        for (i, logit) in logits.iter_mut().enumerate() {
            if !self.current_token_ids.contains(i) {
                *logit = f32::MIN;
            }
        }
        logits
    }

    fn clear(&mut self) {
        self.sampler.reset();
        self.current_token_ids.clear();
    }

    fn clone(&self) -> Box<dyn Transformer> {
        Box::new(Clone::clone(self))
    }
}
