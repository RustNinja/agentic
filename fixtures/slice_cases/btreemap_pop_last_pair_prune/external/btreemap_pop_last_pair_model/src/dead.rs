pub struct DeadBtreemapPopLastPairItem {
    value: String,
}

impl DeadBtreemapPopLastPairItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreemap-pop-last-pair:{}", self.value)
    }
}

pub fn dead_btreemap_pop_last_pair(raw: &str) -> String {
    DeadBtreemapPopLastPairItem::new(raw).render()
}
