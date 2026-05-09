pub struct DeadNonNullAsRefMapItem {
    value: String,
}

impl DeadNonNullAsRefMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-nonnull-as-ref-map:{}", self.value)
    }
}

pub fn dead_nonnull_as_ref_map(raw: &str) -> String {
    DeadNonNullAsRefMapItem::new(raw).render()
}
