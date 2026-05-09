use std::collections::VecDeque;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectVecDequeSource {
    value: String,
}

impl CollectVecDequeSource {
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
        format!("unused-source-collect-vecdeque:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectVecDequeItem {
    value: String,
}

impl CollectVecDequeItem {
    pub fn from_source(source: &CollectVecDequeSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-vecdeque:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-vecdeque:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-vecdeque:{}", self.value)
    }
}

fn collect_vecdeque_sources(raw: &str) -> Vec<CollectVecDequeSource> {
    vec![CollectVecDequeSource::live(), CollectVecDequeSource::new(raw)]
}

pub fn selected_collect_vecdeque(raw: &str) -> String {
    let collected = collect_vecdeque_sources(raw)
        .iter()
        .filter(|source| source.is_live_source())
        .map(CollectVecDequeItem::from_source)
        .collect::<VecDeque<CollectVecDequeItem>>();
    collected
        .iter()
        .map(|item| item.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_collect_vecdeque(raw: &str) -> String {
    CollectVecDequeItem::from_source(&CollectVecDequeSource::new(raw)).dead_method()
}
