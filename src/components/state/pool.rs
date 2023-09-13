use std::sync::{Arc, RwLock};

use anyhow::{Error, Result};
use itertools::Itertools;
use tokio::sync::{mpsc, oneshot};
use web_rwkv::{
    context::Context,
    model::{Model, ModelState},
};

use super::{state::InferState, InferStates};

pub struct InferRequest {
    state: InferState,
    tokens: Vec<u16>,
    callback: oneshot::Sender<Vec<f32>>,
}

type BatchSlots = Arc<RwLock<Vec<Option<InferState>>>>;

struct Slots {
    token_slots: Vec<Vec<u16>>,
    batch_slots: BatchSlots,
    callback_slots: Vec<Option<oneshot::Sender<Vec<f32>>>>,
    pool: Arc<ModelState>,
}

pub struct InferPool {
    max_concurrent: usize,
    batch_size: usize,
    pool: Arc<ModelState>,
    batch_slots: BatchSlots,
    context: Arc<Context>,
    model: Arc<Model<'static>>,
}

impl Slots {
    pub fn new(batch_slots: Arc<RwLock<Vec<Option<InferState>>>>, pool: Arc<ModelState>) -> Self {
        let batch_size = batch_slots.read().unwrap().len();
        Self {
            token_slots: (0..batch_size).map(|_| Vec::new()).collect_vec(),
            batch_slots,
            callback_slots: (0..batch_size).map(|_| None).collect_vec(),
            pool,
        }
    }

    #[inline(always)]
    fn filled_slots(&self) -> usize {
        self.callback_slots.iter().filter(|x| x.is_some()).count()
    }

    #[inline(always)]
    fn empty_slots(&self) -> usize {
        self.callback_slots.iter().filter(|x| x.is_none()).count()
    }

    #[inline(always)]
    fn swap(&mut self, index: usize, state: InferState) {
        let mut writelock = self.batch_slots.write().unwrap();
        if let Some(slot) = std::mem::replace(&mut writelock[index], Some(state)) {
            slot.back_from(&self.pool, index).unwrap();
        };
        writelock[index]
            .as_ref()
            .unwrap()
            .load_to(&self.pool, index)
            .unwrap();
    }

    fn insert(&mut self, request: InferRequest) {
        let InferRequest {
            state,
            tokens,
            callback,
        } = request;

        if self.empty_slots() == 0 {
            panic!("The slot is full!");
        }

        let empty_slots = self
            .callback_slots
            .iter()
            .enumerate()
            .filter(|(_, x)| x.is_none())
            .map(|(x, _)| x)
            .collect::<Vec<_>>();

        let selected_slot = 'slot: {
            let readlock = self.batch_slots.read().unwrap();
            // Try to find an empty slot with a same name
            for index in empty_slots.iter() {
                if let Some(batch_state) = readlock[*index].as_ref() {
                    if &state == batch_state {
                        break 'slot *index;
                    }
                }
            }
            // Try to find an empty slot that is not occupied by any state
            for index in empty_slots.iter() {
                if let None = readlock[*index] {
                    break 'slot *index;
                }
            }
            // Try to find the first empty slot
            empty_slots[0]
        };

        self.token_slots[selected_slot] = tokens;
        self.callback_slots[selected_slot] = Some(callback);
        self.swap(selected_slot, state);
    }

    fn finish(&mut self, index: usize, logits: Vec<f32>) {
        if let Some(sender) = std::mem::replace(&mut self.callback_slots[index], None) {
            sender.send(logits).unwrap();
        } else {
            panic!("Sender is empty!")
        }
    }

    fn infer(&mut self, model: &Model<'static>) {
        // TODO: implement infer logic
        model.run(&mut self.token_slots, &self.pool).unwrap();
    }

    fn load_or_queue(&mut self, mut requests: Vec<InferRequest>, queue: &mut Vec<InferRequest>) {
        for _ in 0..self.empty_slots() {
            if let Some(request) = requests.pop() {
                self.insert(request)
            } else {
                return;
            }
        }
        queue.extend(requests);
    }
}

impl InferPool {
    pub fn new(
        context: Arc<Context>,
        model: Arc<Model<'static>>,
        batch_size: usize,
        max_concurrent: usize,
    ) -> Self {
        let slots = Arc::new(RwLock::new((0..batch_size).map(|_| None).collect_vec()));
        Self {
            max_concurrent,
            batch_size,
            pool: Arc::new(ModelState::new(&context, model.info(), batch_size)),
            context,
            model,
            batch_slots: slots,
        }
    }

    pub fn sync(&self, state_id: &str) -> Result<()> {
        for (index, state) in self
            .batch_slots
            .read()
            .map_err(|_| Error::msg("Lock is poisoned!"))?
            .iter()
            .enumerate()
        {
            if let Some(state) = state.as_ref() {
                if state_id == state.get_id() {
                    state.back_from(&self.pool, index)?;
                    break;
                }
            }
        }
        Ok(())
    }

    async fn infer_loop(&self, mut queue: mpsc::Receiver<Vec<InferRequest>>) {
        let mut slots = Slots::new(self.batch_slots.clone(), self.pool.clone());
        let mut queue_buffer: Vec<InferRequest> = Vec::with_capacity(self.batch_size);
        while let Some(requests) = queue.recv().await {
            slots.load_or_queue(requests, &mut queue_buffer);
        }
    }

    pub async fn start_loop(&self) {}
}
