use bit_set::BitSet;
use ndarray::{Array, Array1, ArrayView1, ArrayViewMut1};
use ordered_float::NotNan;
use rayon::prelude::*;

pub fn argsort(data: ArrayView1<f32>) -> Array1<usize> {
    let mut indices = (0..data.len()).collect::<Vec<_>>();
    indices.par_sort_unstable_by_key(|x| NotNan::new(data[*x]).unwrap());
    Array::from_vec(indices)
}

pub fn sort_by_indices(mut data: ArrayViewMut1<f32>, indices: ArrayView1<usize>) {
    let mut sorted = BitSet::with_capacity(indices.len());
    for idx in 0..data.len() {
        if !sorted.contains(idx) {
            let mut current_idx = idx;
            loop {
                let target_idx = indices[current_idx];
                sorted.insert(current_idx);
                if sorted.contains(target_idx) {
                    break;
                }
                data.swap(current_idx, target_idx);
                current_idx = target_idx;
            }
        }
    }
}
