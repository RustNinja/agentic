pub struct DeadTupleFindMapItem {
    value: String,
}

impl DeadTupleFindMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-tuple-find-map:{}", self.value)
    }
}

pub fn dead_tuple_find_map(raw: &str) -> String {
    DeadTupleFindMapItem::new(raw).render()
}
