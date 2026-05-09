use std::cmp::Ordering;
use std::hash::Hash;

#[derive(Clone, Debug)]
pub struct CollectAnnotatedVecTupleSource {
    value: String,
}

impl CollectAnnotatedVecTupleSource {
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
        format!("unused-source-collect-annotated-vec-tuple:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CollectAnnotatedVecTupleKey {
    value: String,
}

impl CollectAnnotatedVecTupleKey {
    pub fn from_source(source: &CollectAnnotatedVecTupleSource) -> Self {
        Self { value: source.key_seed() }
    }

    pub fn render_key(&self) -> String {
        format!("key-collect-annotated-vec-tuple:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-key-collect-annotated-vec-tuple:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectAnnotatedVecTupleItem {
    value: String,
}

impl CollectAnnotatedVecTupleItem {
    pub fn from_source(source: &CollectAnnotatedVecTupleSource) -> Self {
        Self { value: source.value_seed() }
    }

    pub fn render_label(&self) -> String {
        format!("collect-annotated-vec-tuple:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-annotated-vec-tuple:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-annotated-vec-tuple:{}", self.value)
    }
}

fn collect_annotated_vec_tuple_sources(raw: &str) -> Vec<CollectAnnotatedVecTupleSource> {
    vec![CollectAnnotatedVecTupleSource::live(), CollectAnnotatedVecTupleSource::new(raw)]
}

pub fn selected_collect_annotated_vec_tuple(raw: &str) -> String {
    let collected: Vec<(CollectAnnotatedVecTupleKey, CollectAnnotatedVecTupleItem)> = collect_annotated_vec_tuple_sources(raw)
        .iter()
        .map(|source| (CollectAnnotatedVecTupleKey::from_source(source), CollectAnnotatedVecTupleItem::from_source(source)))
        .collect();
    collected
        .iter()
        .map(|(key, item)| format!("{}={}", key.render_key(), item.render_label()))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_collect_annotated_vec_tuple(raw: &str) -> String {
    CollectAnnotatedVecTupleItem::from_source(&CollectAnnotatedVecTupleSource::new(raw)).dead_method()
}
