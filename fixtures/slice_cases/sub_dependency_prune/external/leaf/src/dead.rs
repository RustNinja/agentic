pub struct DeadLeaf {
    value: String,
}

impl DeadLeaf {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-leaf:{}", self.value)
    }
}

pub fn dead_leaf(value: &str) -> String {
    DeadLeaf::new(value).render()
}

