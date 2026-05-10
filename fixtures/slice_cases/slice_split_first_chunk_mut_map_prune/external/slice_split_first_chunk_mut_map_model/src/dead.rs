pub struct DeadSliceSplitFirstChunkMutMapItem;

pub struct DeadSliceSplitFirstChunkMutMapPayload {
    value: String,
}

impl DeadSliceSplitFirstChunkMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-first-chunk-mut-map:{}", self.value)
    }
}

pub fn dead_slice_split_first_chunk_mut_map(raw: &str) -> String {
    DeadSliceSplitFirstChunkMutMapPayload::new(raw).dead_method()
}
