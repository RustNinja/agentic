pub struct DeadResultTransposeUnwrapMapItem {
    value: String,
}

impl DeadResultTransposeUnwrapMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-result-transpose-unwrap-map:{}", self.value)
    }
}

pub fn dead_result_transpose_unwrap_map(raw: &str) -> String {
    DeadResultTransposeUnwrapMapItem::new(raw).render()
}
