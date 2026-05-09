use std::collections::BTreeSet;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectBTreeSetSource {
    value: String,
}

impl CollectBTreeSetSource {
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
        format!("unused-source-collect-btreeset:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CollectBTreeSetItem {
    value: String,
}

impl CollectBTreeSetItem {
    pub fn from_source(source: &CollectBTreeSetSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-btreeset:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-btreeset:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-btreeset:{}", self.value)
    }
}

fn collect_btreeset_sources(raw: &str) -> Vec<CollectBTreeSetSource> {
    vec![CollectBTreeSetSource::live(), CollectBTreeSetSource::new(raw)]
}

pub fn selected_collect_btreeset(raw: &str) -> String {
    let collected = collect_btreeset_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectBTreeSetItem::from_source)
        .collect::<BTreeSet<CollectBTreeSetItem>>();
    collected
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_collect_btreeset(raw: &str) -> String {
    CollectBTreeSetItem::from_source(&CollectBTreeSetSource::new(raw)).dead_method()
}
