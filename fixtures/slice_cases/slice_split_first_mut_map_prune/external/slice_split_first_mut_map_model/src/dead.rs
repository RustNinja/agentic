pub struct DeadSliceSplitFirstMutMapItem {
    value: String,
}

impl DeadSliceSplitFirstMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-split-first-mut-map:{}", self.value)
    }
}

pub fn dead_slice_split_first_mut_map(raw: &str) -> String {
    DeadSliceSplitFirstMutMapItem::new(raw).render()
}
