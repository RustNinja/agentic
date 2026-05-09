use std::collections::HashSet;
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct CollectHashSetSource {
    value: String,
}

impl CollectHashSetSource {
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
        format!("unused-source-collect-hashset:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CollectHashSetItem {
    value: String,
}

impl CollectHashSetItem {
    pub fn from_source(source: &CollectHashSetSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-hashset:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-hashset:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-hashset:{}", self.value)
    }
}

fn collect_hashset_sources(raw: &str) -> Vec<CollectHashSetSource> {
    vec![CollectHashSetSource::live(), CollectHashSetSource::new(raw)]
}

pub fn selected_collect_hashset(raw: &str) -> String {
    let collected = collect_hashset_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectHashSetItem::from_source)
        .collect::<HashSet<CollectHashSetItem>>();
    collected
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_collect_hashset(raw: &str) -> String {
    CollectHashSetItem::from_source(&CollectHashSetSource::new(raw)).dead_method()
}
