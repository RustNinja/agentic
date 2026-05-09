use std::collections::HashSet;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct HashsetDifferencePayload {
    value: String,
}

impl HashsetDifferencePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashset-difference:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-difference:{}", self.value)
    }
}

fn hashset_difference_left(raw: &str) -> HashSet<HashsetDifferencePayload> {
    let mut payloads = HashSet::new();
    payloads.insert(HashsetDifferencePayload::new(raw));
    payloads.insert(HashsetDifferencePayload::new("shared"));
    payloads
}

fn hashset_difference_right() -> HashSet<HashsetDifferencePayload> {
    let mut payloads = HashSet::new();
    payloads.insert(HashsetDifferencePayload::new("shared"));
    payloads.insert(HashsetDifferencePayload::new("tail"));
    payloads
}

pub fn selected_hashset_difference(raw: &str) -> String {
    let left = hashset_difference_left(raw);
    let right = hashset_difference_right();
    left.difference(&right)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "hashset-difference:missing".to_string())
}

pub fn dead_live_hashset_difference(raw: &str) -> String {
    HashsetDifferencePayload::new(raw).dead_method()
}
