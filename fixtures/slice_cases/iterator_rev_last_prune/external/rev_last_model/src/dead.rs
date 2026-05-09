pub struct DeadRevLastItem {
    value: String,
}

impl DeadRevLastItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-rev-last:{}", self.value)
    }
}

pub fn dead_rev_last(raw: &str) -> String {
    DeadRevLastItem::new(raw).dead_method()
}
