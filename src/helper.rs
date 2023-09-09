#[derive(Debug, Clone)]
pub struct Logits(pub Vec<f32>);

impl Logits {
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

#[derive(Debug, Clone)]
pub struct State(pub Vec<f32>);

impl State {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn to_state(self) -> ! {
        todo!()
    }
}
