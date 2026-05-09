pub struct DeadSliceSelectNthUnstableByItem {
    value: String,
}

impl DeadSliceSelectNthUnstableByItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-select-nth-unstable-by:{}", self.value)
    }
}

pub fn dead_slice_select_nth_unstable_by(raw: &str) -> String {
    DeadSliceSelectNthUnstableByItem::new(raw).dead_method()
}
