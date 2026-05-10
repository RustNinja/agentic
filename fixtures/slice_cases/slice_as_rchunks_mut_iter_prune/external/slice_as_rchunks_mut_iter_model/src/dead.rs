pub struct DeadSliceAsRchunksMutIterItem;

pub struct DeadSliceAsRchunksMutIterPayload {
    value: String,
}

impl DeadSliceAsRchunksMutIterPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-as-rchunks-mut-iter:{}", self.value)
    }
}

pub fn dead_slice_as_rchunks_mut_iter(raw: &str) -> String {
    DeadSliceAsRchunksMutIterPayload::new(raw).dead_method()
}
