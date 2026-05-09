pub struct DeadAsRefTraitMapItem {
    value: String,
}

impl DeadAsRefTraitMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-as-ref-trait-map:{}", self.value)
    }
}

pub fn dead_as_ref_trait_map(raw: &str) -> String {
    DeadAsRefTraitMapItem::new(raw).render()
}
