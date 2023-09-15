use std::sync::Arc;

use crate::{app::AppState, components::InferenceInterruption};
use anyhow::{Error, Result};
use bit_set::BitSet;
use bnf_sampler::{
    grammar::Grammar,
    sampler::{AcceptTokenResult, PossibleTokensResult, Sampler},
    utils::U8ArrayWrapper,
    vocabulary::Vocabulary,
};
use qp_trie::Trie;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_json::Value;

#[derive(Debug, Deserialize, Clone)]
pub struct BNFData {
    grammar: String,
    stack_arena_capacity: usize,
    grammar_stack_arena_capacity: usize,
    start_nonterminal: String,
    stack_to_bytes_cache_enabled: bool,
}

use super::types::Transformer;
#[derive(Debug, Clone)]
pub struct BNFConstraint {
    sampler: Sampler,
    current_token_ids: BitSet,
}

impl BNFConstraint {
    pub fn initialize(state: AppState, data: Option<Value>) -> Result<Box<dyn Transformer>> {
        let vocabulary = state.0.tokenizer.bytes_to_token_index();
        let token_to_id = Trie::from_iter(
            vocabulary
                .iter()
                .map(|(k, v)| (U8ArrayWrapper(k.clone().into_boxed_slice()), *v as u32)),
        );
        let mut id_to_token = FxHashMap::default();
        id_to_token.extend(vocabulary.iter().map(|(k, v)| (*v as u32, k.clone())));
        let mut id_to_token_string = FxHashMap::default();
        id_to_token_string.extend(
            vocabulary
                .iter()
                .map(|(k, v)| (*v as u32, String::from_utf8_lossy(k).to_string())),
        );
        let vocabulary = Arc::new(Vocabulary {
            token_to_id,
            id_to_token,
            id_to_token_string,
        });
        let data = serde_json::from_value::<BNFData>(data.ok_or(Error::msg(
            "Invalid BNFData. Example format:{
                grammar: String,
                stack_arena_capacity: usize,
                grammar_stack_arena_capacity: usize,
                start_nonterminal: String,
                stack_to_bytes_cache_enabled: bool,
            }",
        ))?)?;
        let mut sampler = Sampler::new(
            Grammar::new(
                &data.grammar,
                vocabulary.clone(),
                data.grammar_stack_arena_capacity,
            ),
            data.start_nonterminal,
            vocabulary,
            data.stack_arena_capacity,
            data.stack_to_bytes_cache_enabled,
        );
        let current_token_ids = match sampler.all_possible_next_tokens(None) {
            PossibleTokensResult::Continue(token_ids) => token_ids.clone(),
            _ => unreachable!(),
        };
        Ok(Box::new(BNFConstraint {
            sampler,
            current_token_ids,
        }))
    }
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
        println!("{}",logits.len());
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
        self.current_token_ids = match self.sampler.all_possible_next_tokens(None) {
            PossibleTokensResult::Continue(token_ids) => token_ids.clone(),
            _ => unreachable!(),
        };
    }

    fn clone(&self) -> Box<dyn Transformer> {
        Box::new(Clone::clone(self))
    }
}
