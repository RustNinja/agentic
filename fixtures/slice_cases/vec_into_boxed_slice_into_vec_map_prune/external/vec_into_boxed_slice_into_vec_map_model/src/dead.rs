pub struct DeadVecIntoBoxedSliceIntoVecMapItem;

pub struct DeadVecIntoBoxedSliceIntoVecMapPayload {
    value: String,
}

impl DeadVecIntoBoxedSliceIntoVecMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-into-boxed-slice-into-vec-map:{}", self.value)
    }
}

pub fn dead_vec_into_boxed_slice_into_vec_map(raw: &str) -> String {
    DeadVecIntoBoxedSliceIntoVecMapPayload::new(raw).dead_method()
}
