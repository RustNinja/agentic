use std::collections::HashSet;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct HashsetIntoIterItem {
    value: String,
}

impl HashsetIntoIterItem {
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
        format!("hashset-into-iter:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-into-iter:{}", self.value)
    }
}

fn hashset_into_iter_entries(raw: &str) -> HashSet<HashsetIntoIterItem> {
    let mut entries = HashSet::new();
    entries.insert(HashsetIntoIterItem::live());
    entries.insert(HashsetIntoIterItem::new(raw));
    entries
}

pub fn selected_hashset_into_iter(raw: &str) -> String {
    let entries = hashset_into_iter_entries(raw);
    entries
        .into_iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_hashset_into_iter(raw: &str) -> String {
    HashsetIntoIterItem::new(raw).dead_method()
}
