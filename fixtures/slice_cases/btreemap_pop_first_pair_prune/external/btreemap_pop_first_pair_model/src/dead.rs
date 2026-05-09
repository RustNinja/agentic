pub struct DeadBtreemapPopFirstPairItem {
    value: String,
}

impl DeadBtreemapPopFirstPairItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreemap-pop-first-pair:{}", self.value)
    }
}

pub fn dead_btreemap_pop_first_pair(raw: &str) -> String {
    DeadBtreemapPopFirstPairItem::new(raw).render()
}
