pub struct DeadSliceFirstChunkMutMapItem;

pub struct DeadSliceFirstChunkMutMapPayload {
    value: String,
}

impl DeadSliceFirstChunkMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-first-chunk-mut-map:{}", self.value)
    }
}

pub fn dead_slice_first_chunk_mut_map(raw: &str) -> String {
    DeadSliceFirstChunkMutMapPayload::new(raw).dead_method()
}
