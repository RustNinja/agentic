use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashsetDrainItem {
    value: String,
}

impl HashsetDrainItem {
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
        format!("hashset-drain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-drain:{}", self.value)
    }
}

fn hashset_drain_entries(raw: &str) -> HashSet<HashsetDrainItem> {
    let mut entries = HashSet::new();
    entries.insert(HashsetDrainItem::live());
    entries.insert(HashsetDrainItem::new(raw));
    entries
}

pub fn selected_hashset_drain(raw: &str) -> String {
    let mut entries = hashset_drain_entries(raw);
    entries
        .drain()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_hashset_drain(raw: &str) -> String {
    HashsetDrainItem::new(raw).dead_method()
}
