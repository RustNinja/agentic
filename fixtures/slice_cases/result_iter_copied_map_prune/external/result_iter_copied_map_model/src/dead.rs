pub struct DeadResultIterCopiedMapItem {
    value: String,
}

impl DeadResultIterCopiedMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-iter-copied-map:{}", self.value)
    }
}

pub fn dead_result_iter_copied_map(raw: &str) -> String {
    DeadResultIterCopiedMapItem::new(raw).render()
}
