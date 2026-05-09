pub struct DeadResultAsDerefMapItem {
    value: String,
}

impl DeadResultAsDerefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-as-deref-map:{}", self.value)
    }
}

pub fn dead_result_as_deref_map(raw: &str) -> String {
    DeadResultAsDerefMapItem::new(raw).render()
}
