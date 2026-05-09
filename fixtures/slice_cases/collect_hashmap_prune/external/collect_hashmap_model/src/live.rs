use std::collections::HashMap;
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct CollectHashMapSource {
    value: String,
}

impl CollectHashMapSource {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn live() -> Self {
        Self::new("live")
    }

    pub fn key_seed(&self) -> String {
        format!("key:{}", self.value)
    }

    pub fn value_seed(&self) -> String {
        format!("value:{}", self.value)
    }

    pub fn is_live_source(&self) -> bool {
        self.value.contains("live")
    }

    pub fn unused_source_helper(&self) -> String {
        format!("unused-source-collect-hashmap:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CollectHashMapKey {
    value: String,
}

impl CollectHashMapKey {
    pub fn from_source(source: &CollectHashMapSource) -> Self {
        Self {
            value: source.key_seed(),
        }
    }

    pub fn render_key(&self) -> String {
        format!("key-collect-hashmap:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-key-collect-hashmap:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectHashMapItem {
    value: String,
}

impl CollectHashMapItem {
    pub fn from_source(source: &CollectHashMapSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-hashmap:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-hashmap:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-hashmap:{}", self.value)
    }
}

fn collect_hashmap_sources(raw: &str) -> Vec<CollectHashMapSource> {
    vec![CollectHashMapSource::live(), CollectHashMapSource::new(raw)]
}

pub fn selected_collect_hashmap(raw: &str) -> String {
    let collected = collect_hashmap_sources(raw)
        .iter()
        .map(|source| (CollectHashMapKey::from_source(source), CollectHashMapItem::from_source(source)))
        .collect::<HashMap<CollectHashMapKey, CollectHashMapItem>>();
    collected
        .values()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_collect_hashmap(raw: &str) -> String {
    CollectHashMapItem::from_source(&CollectHashMapSource::new(raw)).dead_method()
}
