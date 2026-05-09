pub struct DeadBtreemapValuesMutItem {
    value: String,
}

impl DeadBtreemapValuesMutItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreemap-values-mut:{}", self.value)
    }
}

pub fn dead_btreemap_values_mut(raw: &str) -> String {
    DeadBtreemapValuesMutItem::new(raw).render()
}
