pub struct DeadSliceChunkByFlattenMapItem;

pub struct DeadSliceChunkByFlattenMapPayload {
    value: String,
}

impl DeadSliceChunkByFlattenMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-chunk-by-flatten-map:{}", self.value)
    }
}

pub fn dead_slice_chunk_by_flatten_map(raw: &str) -> String {
    DeadSliceChunkByFlattenMapPayload::new(raw).dead_method()
}
