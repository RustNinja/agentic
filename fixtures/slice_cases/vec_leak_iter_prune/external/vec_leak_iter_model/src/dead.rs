pub struct DeadVecLeakIterItem {
    value: String,
}

impl DeadVecLeakIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-leak-iter:{}", self.value)
    }
}

pub fn dead_vec_leak_iter(raw: &str) -> String {
    DeadVecLeakIterItem::new(raw).render()
}
