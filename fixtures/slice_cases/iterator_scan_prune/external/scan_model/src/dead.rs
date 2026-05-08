pub struct DeadScanItem {
    value: String,
}

impl DeadScanItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-scan:{}", self.value)
    }
}

pub fn dead_scan(raw: &str) -> String {
    DeadScanItem::new(raw).render()
}
