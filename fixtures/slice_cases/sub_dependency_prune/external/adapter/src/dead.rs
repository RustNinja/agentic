use leaf::dead_leaf;

pub struct DeadAdapter {
    value: String,
}

impl DeadAdapter {
    pub fn new(value: &str) -> Self {
        Self {
            value: value.to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("dead-adapter:{}", self.value)
    }
}

pub fn dead_bridge(value: &str) -> String {
    format!("{}:{}", DeadAdapter::new(value).render(), dead_leaf(value))
}

