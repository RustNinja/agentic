pub struct DeadSliceSplitFirstMapItem {
    value: String,
}

impl DeadSliceSplitFirstMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-split-first-map:{}", self.value)
    }
}

pub fn dead_slice_split_first_map(raw: &str) -> String {
    DeadSliceSplitFirstMapItem::new(raw).render()
}
