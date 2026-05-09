pub struct DeadOptionAsDerefMapItem {
    value: String,
}

impl DeadOptionAsDerefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-as-deref-map:{}", self.value)
    }
}

pub fn dead_option_as_deref_map(raw: &str) -> String {
    DeadOptionAsDerefMapItem::new(raw).render()
}
