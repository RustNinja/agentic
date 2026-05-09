pub struct DeadSliceSplitLastMutMapItem {
    value: String,
}

impl DeadSliceSplitLastMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-split-last-mut-map:{}", self.value)
    }
}

pub fn dead_slice_split_last_mut_map(raw: &str) -> String {
    DeadSliceSplitLastMutMapItem::new(raw).render()
}
