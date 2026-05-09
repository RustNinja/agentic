use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct BTreeMapRetainItem {
    value: String,
}

impl BTreeMapRetainItem {
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
        format!("btreemap-retain:{}", self.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-btreemap-retain:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-retain:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct BTreeMapRetainKey {
    value: String,
}

impl BTreeMapRetainKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn keep(&self) -> bool {
        self.value.contains("live") || self.value.len() > 2
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-key-btreemap-retain:{}", self.value)
    }
}

fn btreemap_retain_entries(raw: &str) -> BTreeMap<BTreeMapRetainKey, BTreeMapRetainItem> {
    let mut map = BTreeMap::new();
    map.insert(BTreeMapRetainKey::new("live"), BTreeMapRetainItem::live());
    map.insert(BTreeMapRetainKey::new("drop"), BTreeMapRetainItem::other());
    map.insert(BTreeMapRetainKey::new(raw), BTreeMapRetainItem::new(raw));
    map
}

pub fn selected_btreemap_retain(raw: &str) -> String {
    let mut entries = btreemap_retain_entries(raw);
    entries.retain(|key, item| key.keep() && item.is_live());
    entries
        .values()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_btreemap_retain(raw: &str) -> String {
    BTreeMapRetainItem::new(raw).dead_method()
}
