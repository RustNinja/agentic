pub struct DeadRegistryEntry {
    raw: String,
}

impl DeadRegistryEntry {
    pub fn new(raw: &str) -> Self {
        Self {
            raw: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-entry:{}", self.raw)
    }
}

pub fn dead_registry_metric(raw: &str) -> String {
    DeadRegistryEntry::new(raw).render()
}
