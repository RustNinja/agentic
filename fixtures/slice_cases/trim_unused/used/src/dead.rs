pub struct DeadRecord {
    value: String,
}

impl DeadRecord {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead:{}", self.value)
    }
}

pub fn format_dead(value: &str) -> String {
    DeadRecord::new(value).render()
}

