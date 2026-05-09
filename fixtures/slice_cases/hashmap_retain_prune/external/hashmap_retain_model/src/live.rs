use std::collections::HashMap;
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct HashMapRetainItem {
    value: String,
}

impl HashMapRetainItem {
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
        format!("hashmap-retain:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-hashmap-retain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-retain:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct HashMapRetainKey {
    value: String,
}

impl HashMapRetainKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn keep(&self) -> bool {
        self.value.contains("live") || self.value.len() > 2
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-key-hashmap-retain:{}", self.value)
    }
}

fn hashmap_retain_entries(raw: &str) -> HashMap<HashMapRetainKey, HashMapRetainItem> {
    let mut map = HashMap::new();
    map.insert(HashMapRetainKey::new("live"), HashMapRetainItem::live());
    map.insert(HashMapRetainKey::new("drop"), HashMapRetainItem::other());
    map.insert(HashMapRetainKey::new(raw), HashMapRetainItem::new(raw));
    map
}

pub fn selected_hashmap_retain(raw: &str) -> String {
    let mut entries = hashmap_retain_entries(raw);
    entries.retain(|key, item| key.keep() && item.is_live());
    entries
        .values()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_hashmap_retain(raw: &str) -> String {
    HashMapRetainItem::new(raw).dead_method()
}
