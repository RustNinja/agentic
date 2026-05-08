#[derive(Clone)]
pub struct DeadWire {
    label: String,
}

impl DeadWire {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.to_string(),
        }
    }

    pub fn render_dead(self) -> String {
        format!("dead-wire:{}", self.label)
    }
}
