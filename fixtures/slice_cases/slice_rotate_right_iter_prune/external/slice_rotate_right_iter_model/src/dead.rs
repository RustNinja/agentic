pub struct DeadSliceRotateRightIterItem {
    value: String,
}

impl DeadSliceRotateRightIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-slice-rotate-right-iter:{}", self.value)
    }
}

pub fn dead_slice_rotate_right_iter(raw: &str) -> String {
    DeadSliceRotateRightIterItem::new(raw).render()
}
