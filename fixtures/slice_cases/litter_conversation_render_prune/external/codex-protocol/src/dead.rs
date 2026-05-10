pub struct DeadEvent {
    value: String,
}

impl DeadEvent {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-event:{}", self.value)
    }
}

pub fn dead_event(value: &str) -> String {
    DeadEvent::new(value).render()
}

