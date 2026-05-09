use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct VecSortUnstableByKeyItem {
    value: String,
}

impl VecSortUnstableByKeyItem {
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
        format!("vec-sort-unstable-by-key:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-vec-sort-unstable-by-key:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-vec-sort-unstable-by-key:{}", self.value)
    }
}

fn vec_sort_unstable_by_key_entries(raw: &str) -> Vec<VecSortUnstableByKeyItem> {
    vec![VecSortUnstableByKeyItem::live(), VecSortUnstableByKeyItem::other(), VecSortUnstableByKeyItem::new(raw)]
}

pub fn selected_vec_sort_unstable_by_key(raw: &str) -> String {
    let mut items = vec_sort_unstable_by_key_entries(raw);
    items.sort_unstable_by_key(|item| item.sort_key());
    items
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_vec_sort_unstable_by_key(raw: &str) -> String {
    VecSortUnstableByKeyItem::new(raw).dead_method()
}
