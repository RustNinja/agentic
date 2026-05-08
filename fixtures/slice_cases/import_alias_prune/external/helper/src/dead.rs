pub struct DeadHelper {
    label: String,
}

impl DeadHelper {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-helper:{}", self.label)
    }
}

pub fn dead_helper(raw: &str) -> DeadHelper {
    DeadHelper::new(raw)
}
