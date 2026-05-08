pub struct DeadEnumStructItem {
    value: String,
}

impl DeadEnumStructItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-enum-struct:{}", self.value)
    }
}

pub fn dead_enum_struct(raw: &str) -> String {
    DeadEnumStructItem::new(raw).render()
}
