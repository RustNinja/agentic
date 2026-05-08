pub struct PartitionItem {
    value: String,
}

impl PartitionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_selected(&self) -> bool {
        self.value.starts_with("live")
    }

    pub fn dead_method(&self) -> String {
        format!("dead-partition:{}", self.value)
    }
}

fn build_items(raw: &str) -> Vec<PartitionItem> {
    raw.split(',').map(PartitionItem::new).collect()
}

pub fn selected_partition(raw: &str) -> String {
    let items = build_items(raw);
    let (selected, rejected): (Vec<_>, Vec<_>) =
        items.iter().partition(|item| item.is_selected());
    format!("partition:{}:{}", selected.len(), rejected.len())
}

pub fn dead_live_partition(raw: &str) -> String {
    PartitionItem::new(raw).dead_method()
}
