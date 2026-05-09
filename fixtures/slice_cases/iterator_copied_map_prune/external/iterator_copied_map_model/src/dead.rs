pub struct DeadIteratorCopiedMapItem {
    value: String,
}

impl DeadIteratorCopiedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iterator-copied-map:{}", self.value)
    }
}

pub fn dead_iterator_copied_map(raw: &str) -> String {
    DeadIteratorCopiedMapItem::new(raw).render()
}
