pub struct DeadBtreemapSplitOffIntoValuesItem {
    value: String,
}

impl DeadBtreemapSplitOffIntoValuesItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreemap-split-off-into-values:{}", self.value)
    }
}

pub fn dead_btreemap_split_off_into_values(raw: &str) -> String {
    DeadBtreemapSplitOffIntoValuesItem::new(raw).render()
}
