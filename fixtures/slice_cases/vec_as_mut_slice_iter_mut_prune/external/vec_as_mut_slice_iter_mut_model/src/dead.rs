pub struct DeadVecAsMutSliceIterMutItem {
    value: String,
}

impl DeadVecAsMutSliceIterMutItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-as-mut-slice-iter-mut:{}", self.value)
    }
}

pub fn dead_vec_as_mut_slice_iter_mut(raw: &str) -> String {
    DeadVecAsMutSliceIterMutItem::new(raw).dead_method()
}
