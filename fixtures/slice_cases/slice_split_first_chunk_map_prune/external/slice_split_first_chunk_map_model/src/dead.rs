pub struct DeadSliceSplitFirstChunkMapItem;

pub struct DeadSliceSplitFirstChunkMapPayload {
    value: String,
}

impl DeadSliceSplitFirstChunkMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-first-chunk-map:{}", self.value)
    }
}

pub fn dead_slice_split_first_chunk_map(raw: &str) -> String {
    DeadSliceSplitFirstChunkMapPayload::new(raw).dead_method()
}
