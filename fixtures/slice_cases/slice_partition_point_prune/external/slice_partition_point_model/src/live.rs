use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct SlicePartitionPointItem {
    value: String,
}

impl SlicePartitionPointItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn other() -> Self {
        Self::new("other")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value.contains("live")
    }

    pub fn bump(&mut self) -> &mut Self {
        self.value.push_str("-live");
        self
    }

    pub fn sort_key(&self) -> usize {
        self.value.len()
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn compare_key(&self, raw: &str) -> Ordering {
        self.value.len().cmp(&raw.len())
    }

    pub fn render_label(&self) -> String {
        format!("slice-partition-point:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-slice-partition-point:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-slice-partition-point:{}", self.value)
    }
}

fn slice_partition_point_entries(raw: &str) -> Vec<SlicePartitionPointItem> {
    vec![SlicePartitionPointItem::live(), SlicePartitionPointItem::other(), SlicePartitionPointItem::new(raw)]
}

pub fn selected_slice_partition_point(raw: &str) -> String {
    let mut items = slice_partition_point_entries(raw);
    items.sort_by_key(|item| item.sort_key());
    let index = items.partition_point(|item| item.sort_key() <= raw.len());
    items
        .get(index.saturating_sub(1))
        .map(|item| item.render_label())
        .unwrap_or_else(|| String::from("missing"))
}

pub fn dead_live_slice_partition_point(raw: &str) -> String {
    SlicePartitionPointItem::new(raw).dead_method()
}
