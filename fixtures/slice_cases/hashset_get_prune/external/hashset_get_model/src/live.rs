use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashsetGetItem {
    value: String,
}

impl HashsetGetItem {
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
        format!("hashset-get:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-get:{}", self.value)
    }
}

fn hashset_get_entries(raw: &str) -> HashSet<HashsetGetItem> {
    let mut entries = HashSet::new();
    entries.insert(HashsetGetItem::live());
    entries.insert(HashsetGetItem::new(raw));
    entries
}

pub fn selected_hashset_get(raw: &str) -> String {
    let entries = hashset_get_entries(raw);
    entries
        .get(&HashsetGetItem::live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "hashset-get:missing".to_string())
}

pub fn dead_live_hashset_get(raw: &str) -> String {
    HashsetGetItem::new(raw).dead_method()
}
