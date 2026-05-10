pub struct DeadBoxedSliceIterMapItem;

pub struct DeadBoxedSliceIterMapPayload {
    value: String,
}

impl DeadBoxedSliceIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-boxed-slice-iter-map:{}", self.value)
    }
}

pub fn dead_boxed_slice_iter_map(raw: &str) -> String {
    DeadBoxedSliceIterMapPayload::new(raw).dead_method()
}
