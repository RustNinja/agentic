use std::cmp::Ordering;

pub struct SortItem {
    value: String,
}

impl SortItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn compare_rank(&self, other: &Self) -> Ordering {
        self.value.len().cmp(&other.value.len())
    }

    pub fn render(&self) -> String {
        format!("sort:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-sort:{}", self.value)
    }
}

fn build_items(raw: &str) -> Vec<SortItem> {
    raw.split(',').map(SortItem::new).collect()
}

pub fn selected_sort_by(raw: &str) -> String {
    let mut items = build_items(raw);
    items.sort_by(|left, right| left.compare_rank(right));
    items
        .iter()
        .map(|item| item.render())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_sort_by(raw: &str) -> String {
    SortItem::new(raw).dead_method()
}
