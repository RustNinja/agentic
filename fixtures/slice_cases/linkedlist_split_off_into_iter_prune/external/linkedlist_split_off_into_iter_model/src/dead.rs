pub struct DeadLinkedlistSplitOffIntoIterItem {
    value: String,
}

impl DeadLinkedlistSplitOffIntoIterItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-linkedlist-split-off-into-iter:{}", self.value)
    }
}

pub fn dead_linkedlist_split_off_into_iter(raw: &str) -> String {
    DeadLinkedlistSplitOffIntoIterItem::new(raw).render()
}
