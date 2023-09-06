
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

pub fn softmax(tensor: Vec<f32>) -> Vec<f32> {
    let tensor = tensor.into_iter();
    let max = tensor.clone().reduce(f32::max).unwrap_or_default();
    let tensor = tensor.map(|x| (x - max).exp());
    let sum: f32 = tensor.clone().sum();
    tensor.map(|x| x / sum).collect()
}
