pub struct DeadResultAsMutMapItem {
    value: String,
}

impl DeadResultAsMutMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-as-mut-map:{}", self.value)
    }
}

pub fn dead_result_as_mut_map(raw: &str) -> String {
    DeadResultAsMutMapItem::new(raw).render()
}
