pub struct DeadBtreemapRangeFindMapItem {
    value: String,
}

impl DeadBtreemapRangeFindMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-range-find-map:{}", self.value)
    }
}

pub fn dead_btreemap_range_find_map(raw: &str) -> String {
    DeadBtreemapRangeFindMapItem::new(raw).dead_method()
}
