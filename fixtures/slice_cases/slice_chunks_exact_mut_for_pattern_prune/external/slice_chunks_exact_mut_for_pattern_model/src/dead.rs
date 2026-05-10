pub struct DeadSliceChunksExactMutForPatternItem;

pub struct DeadSliceChunksExactMutForPatternPayload {
    value: String,
}

impl DeadSliceChunksExactMutForPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-chunks-exact-mut-for-pattern:{}", self.value)
    }
}

pub fn dead_slice_chunks_exact_mut_for_pattern(raw: &str) -> String {
    DeadSliceChunksExactMutForPatternPayload::new(raw).dead_method()
}
