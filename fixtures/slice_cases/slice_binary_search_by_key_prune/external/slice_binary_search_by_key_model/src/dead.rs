pub struct DeadSliceBinarySearchByKeyItem {
    value: String,
}

impl DeadSliceBinarySearchByKeyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-binary-search-by-key:{}", self.value)
    }
}

pub fn dead_slice_binary_search_by_key(raw: &str) -> String {
    DeadSliceBinarySearchByKeyItem::new(raw).dead_method()
}
