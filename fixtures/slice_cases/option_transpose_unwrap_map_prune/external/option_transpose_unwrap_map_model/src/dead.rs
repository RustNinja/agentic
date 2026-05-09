pub struct DeadOptionTransposeUnwrapMapItem {
    value: String,
}

impl DeadOptionTransposeUnwrapMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-transpose-unwrap-map:{}", self.value)
    }
}

pub fn dead_option_transpose_unwrap_map(raw: &str) -> String {
    DeadOptionTransposeUnwrapMapItem::new(raw).render()
}
