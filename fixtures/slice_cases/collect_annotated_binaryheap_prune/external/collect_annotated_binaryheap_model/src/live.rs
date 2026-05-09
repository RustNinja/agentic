use std::collections::BinaryHeap;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedBinaryHeapSource {
    value: String,
}

impl CollectAnnotatedBinaryHeapSource {
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
        format!("unused-source-collect-annotated-binaryheap:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CollectAnnotatedBinaryHeapItem {
    value: String,
}

impl CollectAnnotatedBinaryHeapItem {
    pub fn from_source(source: &CollectAnnotatedBinaryHeapSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-binaryheap:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-binaryheap:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-binaryheap:{}", self.value)
    }
}

fn collect_annotated_binaryheap_sources(raw: &str) -> Vec<CollectAnnotatedBinaryHeapSource> {
    vec![CollectAnnotatedBinaryHeapSource::live(), CollectAnnotatedBinaryHeapSource::new(raw)]
}

pub fn selected_collect_annotated_binaryheap(raw: &str) -> String {
    let collected: BinaryHeap<CollectAnnotatedBinaryHeapItem> = collect_annotated_binaryheap_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectAnnotatedBinaryHeapItem::from_source)
        .collect();
    collected.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("|")
}

pub fn dead_live_collect_annotated_binaryheap(raw: &str) -> String {
    CollectAnnotatedBinaryHeapItem::from_source(&CollectAnnotatedBinaryHeapSource::new(raw)).dead_method()
}
