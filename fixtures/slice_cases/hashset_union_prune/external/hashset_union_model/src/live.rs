use std::collections::HashSet;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct HashsetUnionPayload {
    value: String,
}

impl HashsetUnionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashset-union:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-union:{}", self.value)
    }
}

fn hashset_union_left(raw: &str) -> HashSet<HashsetUnionPayload> {
    let mut payloads = HashSet::new();
    payloads.insert(HashsetUnionPayload::new(raw));
    payloads.insert(HashsetUnionPayload::new("shared"));
    payloads
}

fn hashset_union_right() -> HashSet<HashsetUnionPayload> {
    let mut payloads = HashSet::new();
    payloads.insert(HashsetUnionPayload::new("shared"));
    payloads.insert(HashsetUnionPayload::new("tail"));
    payloads
}

pub fn selected_hashset_union(raw: &str) -> String {
    let left = hashset_union_left(raw);
    let right = hashset_union_right();
    left.union(&right)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "hashset-union:missing".to_string())
}

pub fn dead_live_hashset_union(raw: &str) -> String {
    HashsetUnionPayload::new(raw).dead_method()
}
