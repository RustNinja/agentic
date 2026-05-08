pub struct DeadPartitionItem {
    value: String,
}

impl DeadPartitionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-partition:{}", self.value)
    }
}

pub fn dead_partition(raw: &str) -> String {
    DeadPartitionItem::new(raw).render()
}
