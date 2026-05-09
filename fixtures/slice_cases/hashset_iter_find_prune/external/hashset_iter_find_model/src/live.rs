use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashsetIterFindItem {
    value: String,
}

impl HashsetIterFindItem {
    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn is_live(&self) -> bool {
        self.value == "live"
    }

    pub fn render_label(&self) -> String {
        format!("hashset-iter-find:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-iter-find:{}", self.value)
    }
}

fn hashset_iter_find_entries(raw: &str) -> HashSet<HashsetIterFindItem> {
    let mut entries = HashSet::new();
    entries.insert(HashsetIterFindItem::live());
    entries.insert(HashsetIterFindItem::new(raw));
    entries
}

pub fn selected_hashset_iter_find(raw: &str) -> String {
    let entries = hashset_iter_find_entries(raw);
    entries
        .iter()
        .find(|item| item.is_live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "hashset-iter-find:missing".to_string())
}

pub fn dead_live_hashset_iter_find(raw: &str) -> String {
    HashsetIterFindItem::new(raw).dead_method()
}
