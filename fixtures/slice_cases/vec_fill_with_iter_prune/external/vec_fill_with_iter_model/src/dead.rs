pub struct DeadVecFillWithIterItem {
    value: String,
}

impl DeadVecFillWithIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-fill-with-iter:{}", self.value)
    }
}

pub fn dead_vec_fill_with_iter(raw: &str) -> String {
    DeadVecFillWithIterItem::new(raw).render()
}
