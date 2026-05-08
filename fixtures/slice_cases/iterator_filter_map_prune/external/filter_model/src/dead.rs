pub struct DeadFilterEntry {
    value: String,
}

impl DeadFilterEntry {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-filter-entry:{}", self.value)
    }
}

pub fn dead_filter(raw: &str) -> String {
    DeadFilterEntry::new(raw).render()
}
