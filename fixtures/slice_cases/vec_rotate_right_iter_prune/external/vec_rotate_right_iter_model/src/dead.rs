pub struct DeadVecRotateRightIterItem {
    value: String,
}

impl DeadVecRotateRightIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-rotate-right-iter:{}", self.value)
    }
}

pub fn dead_vec_rotate_right_iter(raw: &str) -> String {
    DeadVecRotateRightIterItem::new(raw).render()
}
