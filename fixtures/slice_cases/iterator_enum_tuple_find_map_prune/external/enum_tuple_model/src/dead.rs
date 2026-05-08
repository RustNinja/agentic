pub struct DeadEnumTupleItem {
    value: String,
}

impl DeadEnumTupleItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-enum-tuple:{}", self.value)
    }
}

pub fn dead_enum_tuple(raw: &str) -> String {
    DeadEnumTupleItem::new(raw).render()
}
