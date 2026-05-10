pub struct DeadSessionHandle {
    value: String,
}

impl DeadSessionHandle {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-session-handle:{}", self.value)
    }
}

pub fn dead_api_report(value: &str) -> String {
    DeadSessionHandle::new(value).render()
}

