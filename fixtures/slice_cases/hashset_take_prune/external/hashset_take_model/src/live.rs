use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashsetTakeItem {
    value: String,
}

impl HashsetTakeItem {
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
        format!("hashset-take:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-take:{}", self.value)
    }
}

fn hashset_take_entries(raw: &str) -> HashSet<HashsetTakeItem> {
    let mut entries = HashSet::new();
    entries.insert(HashsetTakeItem::live());
    entries.insert(HashsetTakeItem::new(raw));
    entries
}

pub fn selected_hashset_take(raw: &str) -> String {
    let mut entries = hashset_take_entries(raw);
    entries
        .take(&HashsetTakeItem::live())
        .map(|item| item.render_label())
        .unwrap_or_else(|| "hashset-take:missing".to_string())
}

pub fn dead_live_hashset_take(raw: &str) -> String {
    HashsetTakeItem::new(raw).dead_method()
}
