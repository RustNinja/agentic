pub struct DeadVecAsSliceChunksItem {
    value: String,
}

impl DeadVecAsSliceChunksItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-as-slice-chunks:{}", self.value)
    }
}

pub fn dead_vec_as_slice_chunks(raw: &str) -> String {
    DeadVecAsSliceChunksItem::new(raw).dead_method()
}
