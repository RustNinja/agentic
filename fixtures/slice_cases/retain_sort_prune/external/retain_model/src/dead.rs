pub struct DeadRetainItem {
    value: String,
}

impl DeadRetainItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-retain-item:{}", self.value)
    }
}

pub fn dead_retain(raw: &str) -> String {
    DeadRetainItem::new(raw).render()
}
