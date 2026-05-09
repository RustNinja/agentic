use std::collections::VecDeque;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedVecDequeSource {
    value: String,
}

impl CollectAnnotatedVecDequeSource {
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
        format!("unused-source-collect-annotated-vecdeque:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectAnnotatedVecDequeItem {
    value: String,
}

impl CollectAnnotatedVecDequeItem {
    pub fn from_source(source: &CollectAnnotatedVecDequeSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-vecdeque:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-vecdeque:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-vecdeque:{}", self.value)
    }
}

fn collect_annotated_vecdeque_sources(raw: &str) -> Vec<CollectAnnotatedVecDequeSource> {
    vec![CollectAnnotatedVecDequeSource::live(), CollectAnnotatedVecDequeSource::new(raw)]
}

pub fn selected_collect_annotated_vecdeque(raw: &str) -> String {
    let collected: VecDeque<CollectAnnotatedVecDequeItem> = collect_annotated_vecdeque_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectAnnotatedVecDequeItem::from_source)
        .collect();
    collected.iter().map(|item| item.render_label()).collect::<Vec<_>>().join("|")
}

pub fn dead_live_collect_annotated_vecdeque(raw: &str) -> String {
    CollectAnnotatedVecDequeItem::from_source(&CollectAnnotatedVecDequeSource::new(raw)).dead_method()
}
