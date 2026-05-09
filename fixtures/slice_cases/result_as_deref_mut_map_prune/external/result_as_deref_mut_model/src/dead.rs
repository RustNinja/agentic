pub struct DeadResultAsDerefMutMapItem {
    value: String,
}

impl DeadResultAsDerefMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-as-deref-mut-map:{}", self.value)
    }
}

pub fn dead_result_as_deref_mut_map(raw: &str) -> String {
    DeadResultAsDerefMutMapItem::new(raw).render()
}
