pub struct DeadVecSortByCachedKeyItem {
    value: String,
}

impl DeadVecSortByCachedKeyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-sort-by-cached-key:{}", self.value)
    }
}

pub fn dead_vec_sort_by_cached_key(raw: &str) -> String {
    DeadVecSortByCachedKeyItem::new(raw).dead_method()
}
