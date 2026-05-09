pub struct DeadSliceRchunksItem {
    value: String,
}

impl DeadSliceRchunksItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rchunks:{}", self.value)
    }
}

pub fn dead_slice_rchunks(raw: &str) -> String {
    DeadSliceRchunksItem::new(raw).dead_method()
}
