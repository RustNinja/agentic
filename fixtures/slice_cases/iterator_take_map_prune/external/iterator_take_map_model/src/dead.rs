pub struct DeadIteratorTakeMapItem {
    value: String,
}

impl DeadIteratorTakeMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iterator-take-map:{}", self.value)
    }
}

pub fn dead_iterator_take_map(raw: &str) -> String {
    DeadIteratorTakeMapItem::new(raw).render()
}
