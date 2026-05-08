pub struct DeadMutexEntry {
    label: String,
}

impl DeadMutexEntry {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-mutex-entry:{}", self.label)
    }
}

pub fn dead_mutex(raw: &str) -> String {
    DeadMutexEntry::new(raw).render()
}
