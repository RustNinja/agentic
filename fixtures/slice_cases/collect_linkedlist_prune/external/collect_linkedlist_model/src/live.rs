use std::collections::LinkedList;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectLinkedListSource {
    value: String,
}

impl CollectLinkedListSource {
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
        format!("unused-source-collect-linkedlist:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectLinkedListItem {
    value: String,
}

impl CollectLinkedListItem {
    pub fn from_source(source: &CollectLinkedListSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-linkedlist:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-linkedlist:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-linkedlist:{}", self.value)
    }
}

fn collect_linkedlist_sources(raw: &str) -> Vec<CollectLinkedListSource> {
    vec![CollectLinkedListSource::live(), CollectLinkedListSource::new(raw)]
}

pub fn selected_collect_linkedlist(raw: &str) -> String {
    let collected = collect_linkedlist_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectLinkedListItem::from_source)
        .collect::<LinkedList<CollectLinkedListItem>>();
    collected
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_collect_linkedlist(raw: &str) -> String {
    CollectLinkedListItem::from_source(&CollectLinkedListSource::new(raw)).dead_method()
}
