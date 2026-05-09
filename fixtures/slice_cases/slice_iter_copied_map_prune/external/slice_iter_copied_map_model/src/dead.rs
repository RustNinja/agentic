pub struct DeadSliceIterCopiedMapItem {
    value: String,
}

impl DeadSliceIterCopiedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-iter-copied-map:{}", self.value)
    }
}

pub fn dead_slice_iter_copied_map(raw: &str) -> String {
    DeadSliceIterCopiedMapItem::new(raw).render()
}
