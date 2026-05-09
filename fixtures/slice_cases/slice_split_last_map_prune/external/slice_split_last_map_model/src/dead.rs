pub struct DeadSliceSplitLastMapItem {
    value: String,
}

impl DeadSliceSplitLastMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-split-last-map:{}", self.value)
    }
}

pub fn dead_slice_split_last_map(raw: &str) -> String {
    DeadSliceSplitLastMapItem::new(raw).render()
}
