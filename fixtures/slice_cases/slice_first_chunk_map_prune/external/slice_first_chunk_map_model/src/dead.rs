pub struct DeadSliceFirstChunkMapItem;

pub struct DeadSliceFirstChunkMapPayload {
    value: String,
}

impl DeadSliceFirstChunkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-first-chunk-map:{}", self.value)
    }
}

pub fn dead_slice_first_chunk_map(raw: &str) -> String {
    DeadSliceFirstChunkMapPayload::new(raw).dead_method()
}
