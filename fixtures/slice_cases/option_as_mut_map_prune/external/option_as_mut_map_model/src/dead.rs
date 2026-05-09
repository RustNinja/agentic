pub struct DeadOptionAsMutMapItem {
    value: String,
}

impl DeadOptionAsMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-as-mut-map:{}", self.value)
    }
}

pub fn dead_option_as_mut_map(raw: &str) -> String {
    DeadOptionAsMutMapItem::new(raw).render()
}
