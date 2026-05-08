pub struct DeadReduceItem {
    value: String,
}

impl DeadReduceItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-reduce:{}", self.value)
    }
}

pub fn dead_reduce(raw: &str) -> String {
    DeadReduceItem::new(raw).render()
}
