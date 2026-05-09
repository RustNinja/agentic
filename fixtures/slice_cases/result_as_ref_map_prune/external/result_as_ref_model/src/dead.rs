pub struct DeadResultAsRefMapItem {
    value: String,
}

impl DeadResultAsRefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-as-ref-map:{}", self.value)
    }
}

pub fn dead_result_as_ref_map(raw: &str) -> String {
    DeadResultAsRefMapItem::new(raw).render()
}
