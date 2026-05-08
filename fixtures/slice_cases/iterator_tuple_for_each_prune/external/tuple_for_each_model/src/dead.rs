pub struct DeadTupleForEachItem {
    value: String,
}

impl DeadTupleForEachItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-tuple-for-each:{}", self.value)
    }
}

pub fn dead_tuple_for_each(raw: &str) -> String {
    DeadTupleForEachItem::new(raw).render()
}
