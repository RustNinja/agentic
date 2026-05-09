pub struct DeadSliceSplitnMutFilterMapItem {
    value: String,
}

impl DeadSliceSplitnMutFilterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-splitn-mut-filter-map:{}", self.value)
    }
}

pub fn dead_slice_splitn_mut_filter_map(raw: &str) -> String {
    DeadSliceSplitnMutFilterMapItem::new(raw).dead_method()
}
