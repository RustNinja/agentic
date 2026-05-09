pub struct DeadSliceSplitAtTailIterItem {
    value: String,
}

impl DeadSliceSplitAtTailIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-split-at-tail-iter:{}", self.value)
    }
}

pub fn dead_slice_split_at_tail_iter(raw: &str) -> String {
    DeadSliceSplitAtTailIterItem::new(raw).render()
}
