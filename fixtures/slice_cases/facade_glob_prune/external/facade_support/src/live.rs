pub struct LiveRecord {
    label: String,
}

impl LiveRecord {
    pub fn new(raw: &str) -> Self {
        Self {
            label: raw.trim().to_string(),
        }
    }

    pub fn render(self) -> String {
        format!("live-facade:{}", self.label)
    }

    pub fn dead_method(self) -> String {
        format!("dead-live-facade:{}", self.label)
    }
}

pub fn build_live(raw: &str) -> LiveRecord {
    LiveRecord::new(raw)
}

pub fn dead_live_factory(raw: &str) -> String {
    LiveRecord::new(raw).dead_method()
}
