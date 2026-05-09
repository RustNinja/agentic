pub struct DeadIteratorStepByMapItem {
    value: String,
}

impl DeadIteratorStepByMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-iterator-step-by-map:{}", self.value)
    }
}

pub fn dead_iterator_step_by_map(raw: &str) -> String {
    DeadIteratorStepByMapItem::new(raw).render()
}
