pub struct DeadVecSortUnstableByKeyItem {
    value: String,
}

impl DeadVecSortUnstableByKeyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-sort-unstable-by-key:{}", self.value)
    }
}

pub fn dead_vec_sort_unstable_by_key(raw: &str) -> String {
    DeadVecSortUnstableByKeyItem::new(raw).dead_method()
}
