pub struct DeadVecdequeAppendIterItem {
    value: String,
}

impl DeadVecdequeAppendIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vecdeque-append-iter:{}", self.value)
    }
}

pub fn dead_vecdeque_append_iter(raw: &str) -> String {
    DeadVecdequeAppendIterItem::new(raw).render()
}
