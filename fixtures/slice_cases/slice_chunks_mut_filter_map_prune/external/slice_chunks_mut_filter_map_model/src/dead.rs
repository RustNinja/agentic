pub struct DeadSliceChunksMutFilterMapItem {
    value: String,
}

impl DeadSliceChunksMutFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-chunks-mut-filter-map:{}", self.value)
    }
}

pub fn dead_slice_chunks_mut_filter_map(raw: &str) -> String {
    DeadSliceChunksMutFilterMapItem::new(raw).dead_method()
}
