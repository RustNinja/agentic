pub struct DeadVecReverseIterItem {
    value: String,
}

impl DeadVecReverseIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-reverse-iter:{}", self.value)
    }
}

pub fn dead_vec_reverse_iter(raw: &str) -> String {
    DeadVecReverseIterItem::new(raw).render()
}
