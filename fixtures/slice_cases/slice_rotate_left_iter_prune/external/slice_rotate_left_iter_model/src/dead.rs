pub struct DeadSliceRotateLeftIterItem {
    value: String,
}

impl DeadSliceRotateLeftIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-rotate-left-iter:{}", self.value)
    }
}

pub fn dead_slice_rotate_left_iter(raw: &str) -> String {
    DeadSliceRotateLeftIterItem::new(raw).render()
}
