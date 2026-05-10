pub struct DeadGeneratedCore {
    label: String,
}

impl DeadGeneratedCore {
    pub fn new(label: &str) -> Self {
        Self {
            label: label.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-core-codegen:{}", self.label)
    }
}

pub fn dead_core_codegen(label: &str) -> String {
    DeadGeneratedCore::new(label).render()
}

