pub struct DeadOptionAsDerefMutMapItem {
    value: String,
}

impl DeadOptionAsDerefMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-as-deref-mut-map:{}", self.value)
    }
}

pub fn dead_option_as_deref_mut_map(raw: &str) -> String {
    DeadOptionAsDerefMutMapItem::new(raw).render()
}
