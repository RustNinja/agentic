pub struct DeadSliceIterMutItem {
    value: String,
}

impl DeadSliceIterMutItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-iter-mut:{}", self.value)
    }
}

pub fn dead_slice_iter_mut(raw: &str) -> String {
    DeadSliceIterMutItem::new(raw).dead_method()
}
