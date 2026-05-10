pub struct DeadSliceAsChunksIterItem;

pub struct DeadSliceAsChunksIterPayload {
    value: String,
}

impl DeadSliceAsChunksIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-as-chunks-iter:{}", self.value)
    }
}

pub fn dead_slice_as_chunks_iter(raw: &str) -> String {
    DeadSliceAsChunksIterPayload::new(raw).dead_method()
}
