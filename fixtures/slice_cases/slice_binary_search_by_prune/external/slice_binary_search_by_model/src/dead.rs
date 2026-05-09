pub struct DeadSliceBinarySearchByItem {
    value: String,
}

impl DeadSliceBinarySearchByItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-binary-search-by:{}", self.value)
    }
}

pub fn dead_slice_binary_search_by(raw: &str) -> String {
    DeadSliceBinarySearchByItem::new(raw).dead_method()
}
