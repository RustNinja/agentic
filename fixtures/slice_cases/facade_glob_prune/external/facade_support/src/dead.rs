pub struct DeadRecord {
    label: String,
}

impl DeadRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.to_string(),
        }
    }

    pub fn render_dead(self) -> String {
        format!("dead-facade:{}", self.label)
    }
}

pub fn dead_factory(raw: &str) -> DeadRecord {
    DeadRecord::new(raw)
}
