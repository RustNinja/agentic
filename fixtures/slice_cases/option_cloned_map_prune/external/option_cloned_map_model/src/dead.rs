pub struct DeadOptionClonedMapItem {
    value: String,
}

impl DeadOptionClonedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-cloned-map:{}", self.value)
    }
}

pub fn dead_option_cloned_map(raw: &str) -> String {
    DeadOptionClonedMapItem::new(raw).render()
}
