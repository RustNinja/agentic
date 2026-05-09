pub struct DeadResultUnwrapOrDefaultMapItem {
    value: String,
}

impl DeadResultUnwrapOrDefaultMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-unwrap-or-default-map:{}", self.value)
    }
}

pub fn dead_result_unwrap_or_default_map(raw: &str) -> String {
    DeadResultUnwrapOrDefaultMapItem::new(raw).render()
}
