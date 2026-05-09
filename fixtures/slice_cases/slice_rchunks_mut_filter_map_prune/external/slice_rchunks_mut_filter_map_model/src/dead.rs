pub struct DeadSliceRchunksMutFilterMapItem {
    value: String,
}

impl DeadSliceRchunksMutFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rchunks-mut-filter-map:{}", self.value)
    }
}

pub fn dead_slice_rchunks_mut_filter_map(raw: &str) -> String {
    DeadSliceRchunksMutFilterMapItem::new(raw).dead_method()
}
