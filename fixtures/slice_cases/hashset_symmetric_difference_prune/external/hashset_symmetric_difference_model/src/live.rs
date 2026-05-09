use std::collections::HashSet;

#[derive(Clone, Eq, PartialEq, Hash)]
pub struct HashsetSymmetricDifferencePayload {
    value: String,
}

impl HashsetSymmetricDifferencePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashset-symmetric-difference:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashset-symmetric-difference:{}", self.value)
    }
}

fn hashset_symmetric_difference_left(raw: &str) -> HashSet<HashsetSymmetricDifferencePayload> {
    let mut payloads = HashSet::new();
    payloads.insert(HashsetSymmetricDifferencePayload::new(raw));
    payloads.insert(HashsetSymmetricDifferencePayload::new("shared"));
    payloads
}

fn hashset_symmetric_difference_right() -> HashSet<HashsetSymmetricDifferencePayload> {
    let mut payloads = HashSet::new();
    payloads.insert(HashsetSymmetricDifferencePayload::new("shared"));
    payloads.insert(HashsetSymmetricDifferencePayload::new("tail"));
    payloads
}

pub fn selected_hashset_symmetric_difference(raw: &str) -> String {
    let left = hashset_symmetric_difference_left(raw);
    let right = hashset_symmetric_difference_right();
    left.symmetric_difference(&right)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "hashset-symmetric-difference:missing".to_string())
}

pub fn dead_live_hashset_symmetric_difference(raw: &str) -> String {
    HashsetSymmetricDifferencePayload::new(raw).dead_method()
}
