use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct CollectVecTupleSource {
    value: String,
}

impl CollectVecTupleSource {
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
        format!("unused-source-collect-vec-tuple:{}", self.value)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CollectVecTupleKey {
    value: String,
}

impl CollectVecTupleKey {
    pub fn from_source(source: &CollectVecTupleSource) -> Self {
        Self {
            value: source.key_seed(),
        }
    }

    pub fn render_key(&self) -> String {
        format!("key-collect-vec-tuple:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-key-collect-vec-tuple:{}", self.value)
    }
}

#[derive(Clone, Debug)]
pub struct CollectVecTupleItem {
    value: String,
}

impl CollectVecTupleItem {
    pub fn from_source(source: &CollectVecTupleSource) -> Self {
        Self {
            value: source.value_seed(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("collect-vec-tuple:{}", self.value)
    }

    pub fn compare(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }

    pub fn unused_helper(&self) -> String {
        format!("unused-collect-vec-tuple:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-collect-vec-tuple:{}", self.value)
    }
}

fn collect_vec_tuple_sources(raw: &str) -> Vec<CollectVecTupleSource> {
    vec![CollectVecTupleSource::live(), CollectVecTupleSource::new(raw)]
}

pub fn selected_collect_vec_tuple(raw: &str) -> String {
    let collected = collect_vec_tuple_sources(raw)
        .iter()
        .map(|source| (CollectVecTupleKey::from_source(source), CollectVecTupleItem::from_source(source)))
        .collect::<Vec<(CollectVecTupleKey, CollectVecTupleItem)>>();
    collected
        .iter()
        .map(|(key, item)| format!("{}={}", key.render_key(), item.render_label()))
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_collect_vec_tuple(raw: &str) -> String {
    CollectVecTupleItem::from_source(&CollectVecTupleSource::new(raw)).dead_method()
}
