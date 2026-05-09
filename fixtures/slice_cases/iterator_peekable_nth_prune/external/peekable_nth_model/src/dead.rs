pub struct DeadPeekableNthItem {
    value: String,
}

impl DeadPeekableNthItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-peekable-nth:{}", self.value)
    }
}

pub fn dead_peekable_nth(raw: &str) -> String {
    DeadPeekableNthItem::new(raw).dead_method()
}
