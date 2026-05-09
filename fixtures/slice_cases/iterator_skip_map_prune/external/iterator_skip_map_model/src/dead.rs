pub struct DeadIteratorSkipMapItem {
    value: String,
}

impl DeadIteratorSkipMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iterator-skip-map:{}", self.value)
    }
}

pub fn dead_iterator_skip_map(raw: &str) -> String {
    DeadIteratorSkipMapItem::new(raw).render()
}
