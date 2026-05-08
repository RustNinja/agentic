pub struct LiveRecord {
    label: String,
}

impl LiveRecord {
    pub fn new(value: &str) -> Self {
        Self {
            label: normalize(value),
        }
    }

    pub fn render(self) -> String {
        format!("live:{}", self.label)
    }

    pub fn dead_method(self) -> String {
        format!("dead:{}", self.label)
    }
}

pub fn format_live(value: &str) -> String {
    LiveRecord::new(value).render()
}

fn normalize(value: &str) -> String {
    value.trim().to_string()
}

pub fn dead_live_helper(value: &str) -> String {
    LiveRecord::new(value).dead_method()
}

