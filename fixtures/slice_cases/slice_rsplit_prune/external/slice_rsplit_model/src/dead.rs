pub struct DeadSliceRsplitItem {
    value: String,
}

impl DeadSliceRsplitItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-rsplit:{}", self.value)
    }
}

pub fn dead_slice_rsplit(raw: &str) -> String {
    DeadSliceRsplitItem::new(raw).dead_method()
}
