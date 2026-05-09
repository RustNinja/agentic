pub struct DeadSliceSortUnstableByItem {
    value: String,
}

impl DeadSliceSortUnstableByItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-sort-unstable-by:{}", self.value)
    }
}

pub fn dead_slice_sort_unstable_by(raw: &str) -> String {
    DeadSliceSortUnstableByItem::new(raw).dead_method()
}
