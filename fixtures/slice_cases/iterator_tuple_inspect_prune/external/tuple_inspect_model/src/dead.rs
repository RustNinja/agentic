pub struct DeadTupleInspectItem {
    value: String,
}

impl DeadTupleInspectItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-tuple-inspect:{}", self.value)
    }
}

pub fn dead_tuple_inspect(raw: &str) -> String {
    DeadTupleInspectItem::new(raw).render()
}
