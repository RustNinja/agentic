pub struct DeadOptionUnwrapOrDefaultMapItem {
    value: String,
}

impl DeadOptionUnwrapOrDefaultMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-option-unwrap-or-default-map:{}", self.value)
    }
}

pub fn dead_option_unwrap_or_default_map(raw: &str) -> String {
    DeadOptionUnwrapOrDefaultMapItem::new(raw).render()
}
