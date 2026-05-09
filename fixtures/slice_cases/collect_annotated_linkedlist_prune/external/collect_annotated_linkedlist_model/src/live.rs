use std::collections::LinkedList;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedLinkedListSource {
    value: String,
}

impl CollectAnnotatedLinkedListSource {
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
        format!("unused-source-collect-annotated-linkedlist:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectAnnotatedLinkedListItem {
    value: String,
}

impl CollectAnnotatedLinkedListItem {
    pub fn from_source(source: &CollectAnnotatedLinkedListSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-linkedlist:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-linkedlist:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-linkedlist:{}", self.value)
    }
}

fn collect_annotated_linkedlist_sources(raw: &str) -> Vec<CollectAnnotatedLinkedListSource> {
    vec![CollectAnnotatedLinkedListSource::live(), CollectAnnotatedLinkedListSource::new(raw)]
}

pub fn selected_collect_annotated_linkedlist(raw: &str) -> String {
    let collected: LinkedList<CollectAnnotatedLinkedListItem> = collect_annotated_linkedlist_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectAnnotatedLinkedListItem::from_source)
        .collect();
    collected.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("|")
}

pub fn dead_live_collect_annotated_linkedlist(raw: &str) -> String {
    CollectAnnotatedLinkedListItem::from_source(&CollectAnnotatedLinkedListSource::new(raw)).dead_method()
}
