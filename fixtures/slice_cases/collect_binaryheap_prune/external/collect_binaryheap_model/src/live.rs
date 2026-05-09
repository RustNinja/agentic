use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectBinaryHeapSource {
    value: String,
}

impl CollectBinaryHeapSource {
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
        format!("unused-source-collect-binaryheap:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CollectBinaryHeapItem {
    value: String,
}

impl CollectBinaryHeapItem {
    pub fn from_source(source: &CollectBinaryHeapSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-binaryheap:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-binaryheap:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-binaryheap:{}", self.value)
    }
}

fn collect_binaryheap_sources(raw: &str) -> Vec<CollectBinaryHeapSource> {
    vec![CollectBinaryHeapSource::live(), CollectBinaryHeapSource::new(raw)]
}

pub fn selected_collect_binaryheap(raw: &str) -> String {
    let collected = collect_binaryheap_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectBinaryHeapItem::from_source)
        .collect::<BinaryHeap<CollectBinaryHeapItem>>();
    collected
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_collect_binaryheap(raw: &str) -> String {
    CollectBinaryHeapItem::from_source(&CollectBinaryHeapSource::new(raw)).dead_method()
}
