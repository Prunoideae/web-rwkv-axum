use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Debug, Clone)]
/// A `BatchRequest` is for locking the GPU infer loop so the run can be more batched.
///
/// When an infer operation starts, the request should request for `infer_size` permits,
/// then the infer loop will wait for the queue to have `max_batch_size.min(total_infer_size)`
/// jobs to start.
///
/// For ultra-long sampling/transforming tasks (which means that the non-infer process
/// is at least 2 times more consuming than other parallel processes), no permit should
/// be requested to ensure run is not stalled.
///
/// But usually that won't happen, probably.
pub struct BatchRequest(Arc<AtomicUsize>);

#[derive(Debug)]
pub struct Permit(usize, BatchRequest);

impl BatchRequest {
    pub fn new() -> Self {
        BatchRequest(Arc::new(AtomicUsize::new(0)))
    }

    pub fn get(&self) -> usize {
        self.0.load(Ordering::Acquire)
    }

    pub fn request(&self, amount: usize) -> Permit {
        self.0.fetch_add(amount, Ordering::Release);
        Permit(amount, self.clone())
    }

    fn release(&self, permit: &Permit) {
        self.0.fetch_sub(permit.0, Ordering::Release);
    }
}

impl Drop for Permit {
    fn drop(&mut self) {
        self.1.release(self)
    }
}
