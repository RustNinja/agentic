pub struct DeadSliceAsChunksMutIterItem;

pub struct DeadSliceAsChunksMutIterPayload {
    value: String,
}

impl DeadSliceAsChunksMutIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-as-chunks-mut-iter:{}", self.value)
    }
}

pub fn dead_slice_as_chunks_mut_iter(raw: &str) -> String {
    DeadSliceAsChunksMutIterPayload::new(raw).dead_method()
}
