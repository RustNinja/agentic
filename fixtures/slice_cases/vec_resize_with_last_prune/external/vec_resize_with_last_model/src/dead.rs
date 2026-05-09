pub struct DeadVecResizeWithLastItem {
    value: String,
}

impl DeadVecResizeWithLastItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-resize-with-last:{}", self.value)
    }
}

pub fn dead_vec_resize_with_last(raw: &str) -> String {
    DeadVecResizeWithLastItem::new(raw).render()
}
