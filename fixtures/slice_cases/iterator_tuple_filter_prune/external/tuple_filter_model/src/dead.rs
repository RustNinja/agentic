pub struct DeadTupleFilterItem {
    value: String,
}

impl DeadTupleFilterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-tuple-filter:{}", self.value)
    }
}

pub fn dead_tuple_filter(raw: &str) -> String {
    DeadTupleFilterItem::new(raw).render()
}
