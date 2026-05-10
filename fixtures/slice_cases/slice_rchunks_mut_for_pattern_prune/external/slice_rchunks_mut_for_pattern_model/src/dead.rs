pub struct DeadSliceRchunksMutForPatternItem;

pub struct DeadSliceRchunksMutForPatternPayload {
    value: String,
}

impl DeadSliceRchunksMutForPatternPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rchunks-mut-for-pattern:{}", self.value)
    }
}

pub fn dead_slice_rchunks_mut_for_pattern(raw: &str) -> String {
    DeadSliceRchunksMutForPatternPayload::new(raw).dead_method()
}
