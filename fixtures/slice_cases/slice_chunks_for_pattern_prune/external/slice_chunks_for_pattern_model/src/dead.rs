pub struct DeadSliceChunksForPatternItem;

pub struct DeadSliceChunksForPatternPayload {
    value: String,
}

impl DeadSliceChunksForPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-chunks-for-pattern:{}", self.value)
    }
}

pub fn dead_slice_chunks_for_pattern(raw: &str) -> String {
    DeadSliceChunksForPatternPayload::new(raw).dead_method()
}
