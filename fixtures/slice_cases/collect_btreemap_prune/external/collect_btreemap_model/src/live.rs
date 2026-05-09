use std::collections::BTreeMap;
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct CollectBTreeMapSource {
    value: String,
}

impl CollectBTreeMapSource {
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
        format!("unused-source-collect-btreemap:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CollectBTreeMapKey {
    value: String,
}

impl CollectBTreeMapKey {
    pub fn from_source(source: &CollectBTreeMapSource) -> Self {
        Self {
            value: source.key_seed(),
        }
    }

    pub fn render_key(&self) -> String {
        format!("key-collect-btreemap:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-key-collect-btreemap:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectBTreeMapItem {
    value: String,
}

impl CollectBTreeMapItem {
    pub fn from_source(source: &CollectBTreeMapSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-btreemap:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-btreemap:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-btreemap:{}", self.value)
    }
}

fn collect_btreemap_sources(raw: &str) -> Vec<CollectBTreeMapSource> {
    vec![CollectBTreeMapSource::live(), CollectBTreeMapSource::new(raw)]
}

pub fn selected_collect_btreemap(raw: &str) -> String {
    let collected = collect_btreemap_sources(raw)
        .iter()
        .map(|source| (CollectBTreeMapKey::from_source(source), CollectBTreeMapItem::from_source(source)))
        .collect::<BTreeMap<CollectBTreeMapKey, CollectBTreeMapItem>>();
    collected
        .values()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_collect_btreemap(raw: &str) -> String {
    CollectBTreeMapItem::from_source(&CollectBTreeMapSource::new(raw)).dead_method()
}
