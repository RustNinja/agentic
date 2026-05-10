pub struct DeadGeneratedProtocol {
    label: String,
}

impl DeadGeneratedProtocol {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-protocol-codegen:{}", self.label)
    }
}

pub fn dead_protocol_codegen_report(label: &str) -> String {
    DeadGeneratedProtocol::new(label).render()
}

