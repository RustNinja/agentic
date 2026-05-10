pub struct DeadWireFrame {
    label: String,
}

impl DeadWireFrame {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-protocol:{}", self.label)
    }
}

pub fn dead_protocol_report(label: &str) -> String {
    DeadWireFrame::new(label).render()
}

