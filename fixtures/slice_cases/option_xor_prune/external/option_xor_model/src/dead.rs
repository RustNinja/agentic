pub struct DeadOptionXorItem {
    value: String,
}

impl DeadOptionXorItem {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.to_string() }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-option-xor:{}", self.value)
    }
}

pub fn dead_option_xor(raw: &str) -> String {
    DeadOptionXorItem::new(raw).dead_method()
}
