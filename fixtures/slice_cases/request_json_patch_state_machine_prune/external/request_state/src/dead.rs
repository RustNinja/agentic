pub struct DeadState {
    label: String,
}

impl DeadState {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-state:{}", self.label)
    }
}

pub fn dead_state_summary(label: &str) -> String {
    DeadState::new(label).render()
}
