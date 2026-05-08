pub struct TuplePartitionKey {
    value: String,
}

impl TuplePartitionKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn keep(&self, value: &TuplePartitionValue) -> bool {
        value.render_label().contains(&self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-partition-key:{}", self.value)
    }
}

pub struct TuplePartitionValue {
    value: String,
}

impl TuplePartitionValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("partition-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-partition-value:{}", self.value)
    }
}

fn tuple_partition_entries(raw: &str) -> Vec<(TuplePartitionKey, TuplePartitionValue)> {
    raw.split(',')
        .map(|part| (TuplePartitionKey::new(part), TuplePartitionValue::new(part)))
        .collect()
}

pub fn selected_tuple_partition(raw: &str) -> String {
    let entries = tuple_partition_entries(raw);
    let (kept, _dropped): (Vec<&(TuplePartitionKey, TuplePartitionValue)>, Vec<_>) =
        entries.iter().partition(|(key, value)| key.keep(value));
    kept.into_iter()
        .map(|(_, value)| value.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_tuple_partition(raw: &str) -> String {
    TuplePartitionKey::new(raw).dead_method()
}
