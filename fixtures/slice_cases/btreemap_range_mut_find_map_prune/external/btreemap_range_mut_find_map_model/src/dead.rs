pub struct DeadBtreemapRangeMutFindMapItem {
    value: String,
}

impl DeadBtreemapRangeMutFindMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-range-mut-find-map:{}", self.value)
    }
}

pub fn dead_btreemap_range_mut_find_map(raw: &str) -> String {
    DeadBtreemapRangeMutFindMapItem::new(raw).dead_method()
}
