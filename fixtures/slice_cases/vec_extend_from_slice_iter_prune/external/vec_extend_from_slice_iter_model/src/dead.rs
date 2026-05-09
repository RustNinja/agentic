pub struct DeadVecExtendFromSliceIterItem {
    value: String,
}

impl DeadVecExtendFromSliceIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-extend-from-slice-iter:{}", self.value)
    }
}

pub fn dead_vec_extend_from_slice_iter(raw: &str) -> String {
    DeadVecExtendFromSliceIterItem::new(raw).render()
}
