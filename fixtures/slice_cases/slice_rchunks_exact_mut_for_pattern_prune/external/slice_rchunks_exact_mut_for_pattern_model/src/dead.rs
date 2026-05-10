pub struct DeadSliceRchunksExactMutForPatternItem;

pub struct DeadSliceRchunksExactMutForPatternPayload {
    value: String,
}

impl DeadSliceRchunksExactMutForPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rchunks-exact-mut-for-pattern:{}", self.value)
    }
}

pub fn dead_slice_rchunks_exact_mut_for_pattern(raw: &str) -> String {
    DeadSliceRchunksExactMutForPatternPayload::new(raw).dead_method()
}
