pub struct DeadTupleMapItem {
    value: String,
}

impl DeadTupleMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-tuple-map:{}", self.value)
    }
}

pub fn dead_tuple_map(raw: &str) -> String {
    DeadTupleMapItem::new(raw).render()
}
