pub struct DeadVecIntoBoxedSliceIterItem {
    value: String,
}

impl DeadVecIntoBoxedSliceIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-into-boxed-slice-iter:{}", self.value)
    }
}

pub fn dead_vec_into_boxed_slice_iter(raw: &str) -> String {
    DeadVecIntoBoxedSliceIterItem::new(raw).render()
}
