pub struct DeadEventApi {
    label: String,
}

impl DeadEventApi {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-api:{}", self.label)
    }
}

pub fn dead_event_api_report(label: &str) -> String {
    DeadEventApi::new(label).render()
}

