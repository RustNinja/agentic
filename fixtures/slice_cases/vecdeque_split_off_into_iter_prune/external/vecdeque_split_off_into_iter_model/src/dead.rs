pub struct DeadVecdequeSplitOffIntoIterItem {
    value: String,
}

impl DeadVecdequeSplitOffIntoIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-vecdeque-split-off-into-iter:{}", self.value)
    }
}

pub fn dead_vecdeque_split_off_into_iter(raw: &str) -> String {
    DeadVecdequeSplitOffIntoIterItem::new(raw).render()
}
