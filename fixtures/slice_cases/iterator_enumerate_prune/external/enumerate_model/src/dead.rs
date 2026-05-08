pub struct DeadEnumerateItem {
    value: String,
}

impl DeadEnumerateItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-enumerate:{}", self.value)
    }
}

pub fn dead_enumerate(raw: &str) -> String {
    DeadEnumerateItem::new(raw).render()
}
