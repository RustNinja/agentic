pub struct DeadVecAppendIterItem {
    value: String,
}

impl DeadVecAppendIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vec-append-iter:{}", self.value)
    }
}

pub fn dead_vec_append_iter(raw: &str) -> String {
    DeadVecAppendIterItem::new(raw).render()
}
