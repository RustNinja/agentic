pub struct DeadSlicePartitionPointItem {
    value: String,
}

impl DeadSlicePartitionPointItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-partition-point:{}", self.value)
    }
}

pub fn dead_slice_partition_point(raw: &str) -> String {
    DeadSlicePartitionPointItem::new(raw).dead_method()
}
