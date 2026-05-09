pub struct DeadSliceChunksItem {
    value: String,
}

impl DeadSliceChunksItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-chunks:{}", self.value)
    }
}

pub fn dead_slice_chunks(raw: &str) -> String {
    DeadSliceChunksItem::new(raw).dead_method()
}
