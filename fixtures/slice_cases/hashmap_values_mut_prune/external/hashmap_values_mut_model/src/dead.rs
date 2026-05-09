pub struct DeadHashmapValuesMutItem {
    value: String,
}

impl DeadHashmapValuesMutItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-hashmap-values-mut:{}", self.value)
    }
}

pub fn dead_hashmap_values_mut(raw: &str) -> String {
    DeadHashmapValuesMutItem::new(raw).render()
}
