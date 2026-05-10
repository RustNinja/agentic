pub struct DeadVecSortByMapItem {
    value: String,
}

impl DeadVecSortByMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-sort-by-map:{}", self.value)
    }
}

pub fn dead_vec_sort_by_map(raw: &str) -> String {
    DeadVecSortByMapItem::new(raw).dead_method()
}
