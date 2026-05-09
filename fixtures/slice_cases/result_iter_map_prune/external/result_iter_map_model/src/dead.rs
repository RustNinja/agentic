pub struct DeadResultIterMapItem {
    value: String,
}

impl DeadResultIterMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-iter-map:{}", self.value)
    }
}

pub fn dead_result_iter_map(raw: &str) -> String {
    DeadResultIterMapItem::new(raw).render()
}
