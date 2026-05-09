pub struct DeadSliceReverseIterItem {
    value: String,
}

impl DeadSliceReverseIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-reverse-iter:{}", self.value)
    }
}

pub fn dead_slice_reverse_iter(raw: &str) -> String {
    DeadSliceReverseIterItem::new(raw).render()
}
