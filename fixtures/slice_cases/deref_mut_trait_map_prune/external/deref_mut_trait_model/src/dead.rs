pub struct DeadDerefMutTraitMapItem {
    value: String,
}

impl DeadDerefMutTraitMapItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-deref-mut-trait-map:{}", self.value)
    }
}

pub fn dead_deref_mut_trait_map(raw: &str) -> String {
    DeadDerefMutTraitMapItem::new(raw).render()
}
