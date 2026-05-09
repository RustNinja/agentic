pub struct DeadSliceChunksExactItem {
    value: String,
}

impl DeadSliceChunksExactItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-chunks-exact:{}", self.value)
    }
}

pub fn dead_slice_chunks_exact(raw: &str) -> String {
    DeadSliceChunksExactItem::new(raw).dead_method()
}
