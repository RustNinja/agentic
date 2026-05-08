pub struct DeadZipItem {
    value: String,
}

impl DeadZipItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-zip:{}", self.value)
    }
}

pub fn dead_zip(raw: &str) -> String {
    DeadZipItem::new(raw).render()
}
