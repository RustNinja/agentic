use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashsetReplaceItem {
    value: String,
}

impl HashsetReplaceItem {
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
        format!("hashset-replace:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-replace:{}", self.value)
    }
}

fn hashset_replace_entries(raw: &str) -> HashSet<HashsetReplaceItem> {
    let mut entries = HashSet::new();
    entries.insert(HashsetReplaceItem::live());
    entries.insert(HashsetReplaceItem::new(raw));
    entries
}

pub fn selected_hashset_replace(raw: &str) -> String {
    let mut entries = hashset_replace_entries(raw);
    entries
        .replace(HashsetReplaceItem::live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "hashset-replace:missing".to_string())
}

pub fn dead_live_hashset_replace(raw: &str) -> String {
    HashsetReplaceItem::new(raw).dead_method()
}
