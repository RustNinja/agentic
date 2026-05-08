pub struct UnusedRecord {
    value: String,
}

impl UnusedRecord {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("unused:{}", self.value)
    }
}

pub fn format_dead(value: &str) -> String {
    UnusedRecord::new(value).render()
}

