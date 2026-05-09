pub struct DeadHashsetUnionItem {
    value: String,
}

impl DeadHashsetUnionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-hashset-union:{}", self.value)
    }
}

pub fn dead_hashset_union(raw: &str) -> String {
    DeadHashsetUnionItem::new(raw).render()
}
