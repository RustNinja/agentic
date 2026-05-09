pub struct DeadResultClonedMapItem {
    value: String,
}

impl DeadResultClonedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-cloned-map:{}", self.value)
    }
}

pub fn dead_result_cloned_map(raw: &str) -> String {
    DeadResultClonedMapItem::new(raw).render()
}
