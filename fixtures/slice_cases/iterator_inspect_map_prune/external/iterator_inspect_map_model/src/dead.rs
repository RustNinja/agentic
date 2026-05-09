pub struct DeadIteratorInspectMapItem {
    value: String,
}

impl DeadIteratorInspectMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iterator-inspect-map:{}", self.value)
    }
}

pub fn dead_iterator_inspect_map(raw: &str) -> String {
    DeadIteratorInspectMapItem::new(raw).render()
}
