use std::collections::HashSet;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct HashsetIntersectionPayload {
    value: String,
}

impl HashsetIntersectionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashset-intersection:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-intersection:{}", self.value)
    }
}

fn hashset_intersection_left(raw: &str) -> HashSet<HashsetIntersectionPayload> {
    let mut payloads = HashSet::new();
    payloads.insert(HashsetIntersectionPayload::new(raw));
    payloads.insert(HashsetIntersectionPayload::new("shared"));
    payloads
}

fn hashset_intersection_right() -> HashSet<HashsetIntersectionPayload> {
    let mut payloads = HashSet::new();
    payloads.insert(HashsetIntersectionPayload::new("shared"));
    payloads.insert(HashsetIntersectionPayload::new("tail"));
    payloads
}

pub fn selected_hashset_intersection(raw: &str) -> String {
    let left = hashset_intersection_left(raw);
    let right = hashset_intersection_right();
    left.intersection(&right)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "hashset-intersection:missing".to_string())
}

pub fn dead_live_hashset_intersection(raw: &str) -> String {
    HashsetIntersectionPayload::new(raw).dead_method()
}
