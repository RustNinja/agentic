pub struct DeadForEachEvent {
    value: String,
}

impl DeadForEachEvent {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-for-each:{}", self.value)
    }
}

pub fn dead_for_each(raw: &str) -> String {
    DeadForEachEvent::new(raw).render()
}
