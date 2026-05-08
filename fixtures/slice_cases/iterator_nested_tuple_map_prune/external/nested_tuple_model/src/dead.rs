pub struct DeadNestedTupleItem {
    value: String,
}

impl DeadNestedTupleItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-nested-tuple:{}", self.value)
    }
}

pub fn dead_nested_tuple_map(raw: &str) -> String {
    DeadNestedTupleItem::new(raw).render()
}
