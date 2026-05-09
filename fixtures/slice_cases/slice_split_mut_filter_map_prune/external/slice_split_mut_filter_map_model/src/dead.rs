pub struct DeadSliceSplitMutFilterMapItem {
    value: String,
}

impl DeadSliceSplitMutFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-mut-filter-map:{}", self.value)
    }
}

pub fn dead_slice_split_mut_filter_map(raw: &str) -> String {
    DeadSliceSplitMutFilterMapItem::new(raw).dead_method()
}
