pub struct DeadAsMutTraitMapItem {
    value: String,
}

impl DeadAsMutTraitMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-as-mut-trait-map:{}", self.value)
    }
}

pub fn dead_as_mut_trait_map(raw: &str) -> String {
    DeadAsMutTraitMapItem::new(raw).render()
}
