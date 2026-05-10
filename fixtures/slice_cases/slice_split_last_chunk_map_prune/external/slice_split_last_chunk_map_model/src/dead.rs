pub struct DeadSliceSplitLastChunkMapItem;

pub struct DeadSliceSplitLastChunkMapPayload {
    value: String,
}

impl DeadSliceSplitLastChunkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-last-chunk-map:{}", self.value)
    }
}

pub fn dead_slice_split_last_chunk_map(raw: &str) -> String {
    DeadSliceSplitLastChunkMapPayload::new(raw).dead_method()
}
