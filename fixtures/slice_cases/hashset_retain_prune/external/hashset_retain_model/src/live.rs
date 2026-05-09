use std::collections::HashSet;
use std::cmp::Ordering;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HashSetRetainItem {
    value: String,
}

impl HashSetRetainItem {
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

    pub fn render_label(&self) -> String {
        format!("hashset-retain:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-hashset-retain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-retain:{}", self.value)
    }
}

fn hashset_retain_entries(raw: &str) -> HashSet<HashSetRetainItem> {
    let mut set = HashSet::new();
    set.insert(HashSetRetainItem::live());
    set.insert(HashSetRetainItem::other());
    set.insert(HashSetRetainItem::new(raw));
    set
}

pub fn selected_hashset_retain(raw: &str) -> String {
    let mut entries = hashset_retain_entries(raw);
    entries.retain(|item| item.is_live());
    entries
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_hashset_retain(raw: &str) -> String {
    HashSetRetainItem::new(raw).dead_method()
}
