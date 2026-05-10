pub struct DeadReader {
    label: String,
}

impl DeadReader {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-reader:{}", self.label)
    }
}

pub fn dead_dyn_summary(raw: &str) -> String {
    DeadReader::new(raw).render()
}

