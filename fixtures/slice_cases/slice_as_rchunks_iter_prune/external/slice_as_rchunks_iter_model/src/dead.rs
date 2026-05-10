pub struct DeadSliceAsRchunksIterItem;

pub struct DeadSliceAsRchunksIterPayload {
    value: String,
}

impl DeadSliceAsRchunksIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-as-rchunks-iter:{}", self.value)
    }
}

pub fn dead_slice_as_rchunks_iter(raw: &str) -> String {
    DeadSliceAsRchunksIterPayload::new(raw).dead_method()
}
