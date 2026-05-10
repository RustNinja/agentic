pub struct DeadSliceChunkByMutFlattenMapItem;

pub struct DeadSliceChunkByMutFlattenMapPayload {
    value: String,
}

impl DeadSliceChunkByMutFlattenMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-chunk-by-mut-flatten-map:{}", self.value)
    }
}

pub fn dead_slice_chunk_by_mut_flatten_map(raw: &str) -> String {
    DeadSliceChunkByMutFlattenMapPayload::new(raw).dead_method()
}
