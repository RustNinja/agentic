pub struct DeadTupleStructItem {
    value: String,
}

impl DeadTupleStructItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-tuple-struct:{}", self.value)
    }
}

pub fn dead_tuple_struct(raw: &str) -> String {
    DeadTupleStructItem::new(raw).render()
}
