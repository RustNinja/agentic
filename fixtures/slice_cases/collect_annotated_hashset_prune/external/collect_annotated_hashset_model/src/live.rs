use std::collections::HashSet;
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedHashSetSource {
    value: String,
}

impl CollectAnnotatedHashSetSource {
    pub fn new(raw: &str) -> Self {
        Self { value: raw.trim().to_string() }
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
        format!("unused-source-collect-annotated-hashset:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CollectAnnotatedHashSetItem {
    value: String,
}

impl CollectAnnotatedHashSetItem {
    pub fn from_source(source: &CollectAnnotatedHashSetSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-hashset:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-hashset:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-hashset:{}", self.value)
    }
}

fn collect_annotated_hashset_sources(raw: &str) -> Vec<CollectAnnotatedHashSetSource> {
    vec![CollectAnnotatedHashSetSource::live(), CollectAnnotatedHashSetSource::new(raw)]
}

pub fn selected_collect_annotated_hashset(raw: &str) -> String {
    let collected: HashSet<CollectAnnotatedHashSetItem> = collect_annotated_hashset_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectAnnotatedHashSetItem::from_source)
        .collect();
    collected.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("|")
}

pub fn dead_live_collect_annotated_hashset(raw: &str) -> String {
    CollectAnnotatedHashSetItem::from_source(&CollectAnnotatedHashSetSource::new(raw)).dead_method()
}
