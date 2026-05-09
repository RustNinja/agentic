pub struct DeadVecAsSliceIterItem {
    value: String,
}

impl DeadVecAsSliceIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-as-slice-iter:{}", self.value)
    }
}

pub fn dead_vec_as_slice_iter(raw: &str) -> String {
    DeadVecAsSliceIterItem::new(raw).dead_method()
}
