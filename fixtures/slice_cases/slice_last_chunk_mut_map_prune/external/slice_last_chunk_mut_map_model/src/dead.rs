pub struct DeadSliceLastChunkMutMapItem;

pub struct DeadSliceLastChunkMutMapPayload {
    value: String,
}

impl DeadSliceLastChunkMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-last-chunk-mut-map:{}", self.value)
    }
}

pub fn dead_slice_last_chunk_mut_map(raw: &str) -> String {
    DeadSliceLastChunkMutMapPayload::new(raw).dead_method()
}
