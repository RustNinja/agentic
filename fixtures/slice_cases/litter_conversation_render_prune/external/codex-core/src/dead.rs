pub struct DeadState {
    value: String,
}

impl DeadState {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-state:{}", self.value)
    }
}

pub fn dead_state(value: &str) -> String {
    DeadState::new(value).render()
}

