pub struct DeadSliceSelectNthUnstableByKeyItem {
    value: String,
}

impl DeadSliceSelectNthUnstableByKeyItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-select-nth-unstable-by-key:{}", self.value)
    }
}

pub fn dead_slice_select_nth_unstable_by_key(raw: &str) -> String {
    DeadSliceSelectNthUnstableByKeyItem::new(raw).dead_method()
}
