pub struct DeadSliceSplitLastChunkMutMapItem;

pub struct DeadSliceSplitLastChunkMutMapPayload {
    value: String,
}

impl DeadSliceSplitLastChunkMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-last-chunk-mut-map:{}", self.value)
    }
}

pub fn dead_slice_split_last_chunk_mut_map(raw: &str) -> String {
    DeadSliceSplitLastChunkMutMapPayload::new(raw).dead_method()
}
