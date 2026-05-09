pub struct DeadVecRotateLeftIterItem {
    value: String,
}

impl DeadVecRotateLeftIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-rotate-left-iter:{}", self.value)
    }
}

pub fn dead_vec_rotate_left_iter(raw: &str) -> String {
    DeadVecRotateLeftIterItem::new(raw).render()
}
