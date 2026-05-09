use std::collections::HashMap;
use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedHashMapSource {
    value: String,
}

impl CollectAnnotatedHashMapSource {
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
        format!("unused-source-collect-annotated-hashmap:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CollectAnnotatedHashMapKey {
    value: String,
}

impl CollectAnnotatedHashMapKey {
    pub fn from_source(source: &CollectAnnotatedHashMapSource) -> Self {
        Self { value: source.key_seed() }
    }

    pub fn render_key(&self) -> String {
        format!("key-collect-annotated-hashmap:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-key-collect-annotated-hashmap:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectAnnotatedHashMapItem {
    value: String,
}

impl CollectAnnotatedHashMapItem {
    pub fn from_source(source: &CollectAnnotatedHashMapSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-hashmap:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-hashmap:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-hashmap:{}", self.value)
    }
}

fn collect_annotated_hashmap_sources(raw: &str) -> Vec<CollectAnnotatedHashMapSource> {
    vec![CollectAnnotatedHashMapSource::live(), CollectAnnotatedHashMapSource::new(raw)]
}

pub fn selected_collect_annotated_hashmap(raw: &str) -> String {
    let collected: HashMap<CollectAnnotatedHashMapKey, CollectAnnotatedHashMapItem> = collect_annotated_hashmap_sources(raw)
        .iter()
        .map(|source| (CollectAnnotatedHashMapKey::from_source(source), CollectAnnotatedHashMapItem::from_source(source)))
        .collect();
    collected.values().map(|item| item.render_label()).collect::<Vec<_>>().join("|")
}

pub fn dead_live_collect_annotated_hashmap(raw: &str) -> String {
    CollectAnnotatedHashMapItem::from_source(&CollectAnnotatedHashMapSource::new(raw)).dead_method()
}
