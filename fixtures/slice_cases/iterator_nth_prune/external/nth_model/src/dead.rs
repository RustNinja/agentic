pub struct DeadNthItem {
    value: String,
}

impl DeadNthItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-nth:{}", self.value)
    }
}

pub fn dead_nth(raw: &str) -> String {
    DeadNthItem::new(raw).dead_method()
}
