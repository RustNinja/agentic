use std::collections::BTreeMap;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedBTreeMapSource {
    value: String,
}

impl CollectAnnotatedBTreeMapSource {
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
        format!("unused-source-collect-annotated-btreemap:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CollectAnnotatedBTreeMapKey {
    value: String,
}

impl CollectAnnotatedBTreeMapKey {
    pub fn from_source(source: &CollectAnnotatedBTreeMapSource) -> Self {
        Self { value: source.key_seed() }
    }

    pub fn render_key(&self) -> String {
        format!("key-collect-annotated-btreemap:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-key-collect-annotated-btreemap:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectAnnotatedBTreeMapItem {
    value: String,
}

impl CollectAnnotatedBTreeMapItem {
    pub fn from_source(source: &CollectAnnotatedBTreeMapSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-btreemap:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-btreemap:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-btreemap:{}", self.value)
    }
}

fn collect_annotated_btreemap_sources(raw: &str) -> Vec<CollectAnnotatedBTreeMapSource> {
    vec![CollectAnnotatedBTreeMapSource::live(), CollectAnnotatedBTreeMapSource::new(raw)]
}

pub fn selected_collect_annotated_btreemap(raw: &str) -> String {
    let collected: BTreeMap<CollectAnnotatedBTreeMapKey, CollectAnnotatedBTreeMapItem> = collect_annotated_btreemap_sources(raw)
        .iter()
        .map(|source| (CollectAnnotatedBTreeMapKey::from_source(source), CollectAnnotatedBTreeMapItem::from_source(source)))
        .collect();
    collected.values().map(|item| item.render_label()).collect::<Vec<_>>().join("|")
}

pub fn dead_live_collect_annotated_btreemap(raw: &str) -> String {
    CollectAnnotatedBTreeMapItem::from_source(&CollectAnnotatedBTreeMapSource::new(raw)).dead_method()
}
