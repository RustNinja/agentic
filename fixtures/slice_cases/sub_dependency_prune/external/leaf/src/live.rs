pub struct LeafRecord {
    label: String,
}

impl LeafRecord {
    pub fn new(value: &str) -> Self {
        Self {
            label: normalize(value),
        }
    }

    pub fn render(self) -> String {
        format!("leaf:{}", self.label)
    }

    pub fn dead_method(self) -> String {
        format!("dead-leaf:{}", self.label)
    }
}

pub fn make_leaf(value: &str) -> LeafRecord {
    LeafRecord::new(value)
}

fn normalize(value: &str) -> String {
    value.trim().to_string()
}

pub fn dead_live_leaf(value: &str) -> String {
    LeafRecord::new(value).dead_method()
}

