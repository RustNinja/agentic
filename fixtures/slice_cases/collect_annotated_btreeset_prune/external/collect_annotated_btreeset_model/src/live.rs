use std::collections::BTreeSet;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedBTreeSetSource {
    value: String,
}

impl CollectAnnotatedBTreeSetSource {
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
        format!("unused-source-collect-annotated-btreeset:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CollectAnnotatedBTreeSetItem {
    value: String,
}

impl CollectAnnotatedBTreeSetItem {
    pub fn from_source(source: &CollectAnnotatedBTreeSetSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-btreeset:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-btreeset:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-btreeset:{}", self.value)
    }
}

fn collect_annotated_btreeset_sources(raw: &str) -> Vec<CollectAnnotatedBTreeSetSource> {
    vec![CollectAnnotatedBTreeSetSource::live(), CollectAnnotatedBTreeSetSource::new(raw)]
}

pub fn selected_collect_annotated_btreeset(raw: &str) -> String {
    let collected: BTreeSet<CollectAnnotatedBTreeSetItem> = collect_annotated_btreeset_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectAnnotatedBTreeSetItem::from_source)
        .collect();
    collected.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("|")
}

pub fn dead_live_collect_annotated_btreeset(raw: &str) -> String {
    CollectAnnotatedBTreeSetItem::from_source(&CollectAnnotatedBTreeSetSource::new(raw)).dead_method()
}
