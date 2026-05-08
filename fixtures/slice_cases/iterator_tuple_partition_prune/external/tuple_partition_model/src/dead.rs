pub struct DeadTuplePartitionItem {
    value: String,
}

impl DeadTuplePartitionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn render(&self) -> String {
        format!("dead-tuple-partition:{}", self.value)
    }
}

pub fn dead_tuple_partition(raw: &str) -> String {
    DeadTuplePartitionItem::new(raw).render()
}
