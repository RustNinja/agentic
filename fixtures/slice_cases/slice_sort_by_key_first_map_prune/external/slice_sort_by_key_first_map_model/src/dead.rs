pub struct DeadSliceSortByKeyFirstMapItem {
    value: String,
}

impl DeadSliceSortByKeyFirstMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-sort-by-key-first-map:{}", self.value)
    }
}

pub fn dead_slice_sort_by_key_first_map(raw: &str) -> String {
    DeadSliceSortByKeyFirstMapItem::new(raw).dead_method()
}
