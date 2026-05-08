pub struct DeadEnumIfLetItem {
    value: String,
}

impl DeadEnumIfLetItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-enum-if-let:{}", self.value)
    }
}

pub fn dead_enum_if_let(raw: &str) -> String {
    DeadEnumIfLetItem::new(raw).render()
}
