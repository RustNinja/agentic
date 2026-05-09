pub struct DeadSliceSplitAtMutTailIterItem {
    value: String,
}

impl DeadSliceSplitAtMutTailIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-split-at-mut-tail-iter:{}", self.value)
    }
}

pub fn dead_slice_split_at_mut_tail_iter(raw: &str) -> String {
    DeadSliceSplitAtMutTailIterItem::new(raw).render()
}
