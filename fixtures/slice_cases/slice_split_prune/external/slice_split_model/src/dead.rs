pub struct DeadSliceSplitItem {
    value: String,
}

impl DeadSliceSplitItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split:{}", self.value)
    }
}

pub fn dead_slice_split(raw: &str) -> String {
    DeadSliceSplitItem::new(raw).dead_method()
}
