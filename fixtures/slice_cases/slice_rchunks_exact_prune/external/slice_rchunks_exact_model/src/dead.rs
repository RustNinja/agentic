pub struct DeadSliceRchunksExactItem {
    value: String,
}

impl DeadSliceRchunksExactItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rchunks-exact:{}", self.value)
    }
}

pub fn dead_slice_rchunks_exact(raw: &str) -> String {
    DeadSliceRchunksExactItem::new(raw).dead_method()
}
