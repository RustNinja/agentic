pub struct DeadSliceRchunksForPatternItem;

pub struct DeadSliceRchunksForPatternPayload {
    value: String,
}

impl DeadSliceRchunksForPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rchunks-for-pattern:{}", self.value)
    }
}

pub fn dead_slice_rchunks_for_pattern(raw: &str) -> String {
    DeadSliceRchunksForPatternPayload::new(raw).dead_method()
}
