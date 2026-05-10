pub struct DeadSliceLastChunkMapItem;

pub struct DeadSliceLastChunkMapPayload {
    value: String,
}

impl DeadSliceLastChunkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-last-chunk-map:{}", self.value)
    }
}

pub fn dead_slice_last_chunk_map(raw: &str) -> String {
    DeadSliceLastChunkMapPayload::new(raw).dead_method()
}
