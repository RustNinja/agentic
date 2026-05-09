pub struct DeadBtreesetSplitOffIntoIterItem {
    value: String,
}

impl DeadBtreesetSplitOffIntoIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-btreeset-split-off-into-iter:{}", self.value)
    }
}

pub fn dead_btreeset_split_off_into_iter(raw: &str) -> String {
    DeadBtreesetSplitOffIntoIterItem::new(raw).render()
}
