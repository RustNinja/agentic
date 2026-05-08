pub struct DeadFold {
    value: String,
}

impl DeadFold {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-fold:{}", self.value)
    }
}

pub fn dead_fold(raw: &str) -> String {
    DeadFold::new(raw).render()
}
