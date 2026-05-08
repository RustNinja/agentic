pub struct DedupItem {
    value: String,
}

impl DedupItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn same_group(&self, other: &Self) -> bool {
        self.value.chars().next() == other.value.chars().next()
    }

    pub fn render(&self) -> String {
        format!("dedup:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-dedup:{}", self.value)
    }
}

fn build_items(raw: &str) -> Vec<DedupItem> {
    raw.split(',').map(DedupItem::new).collect()
}

pub fn selected_dedup_by(raw: &str) -> String {
    let mut items = build_items(raw);
    items.dedup_by(|left, right| left.same_group(right));
    items
        .iter()
        .map(|item| item.render())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_dedup_by(raw: &str) -> String {
    DedupItem::new(raw).dead_method()
}
