pub struct DeadSliceSplitInclusiveItem {
    value: String,
}

impl DeadSliceSplitInclusiveItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-split-inclusive:{}", self.value)
    }
}

pub fn dead_slice_split_inclusive(raw: &str) -> String {
    DeadSliceSplitInclusiveItem::new(raw).dead_method()
}
