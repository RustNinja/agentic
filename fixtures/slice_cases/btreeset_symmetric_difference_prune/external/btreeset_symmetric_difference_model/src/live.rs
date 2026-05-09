use std::collections::BTreeSet;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreesetSymmetricDifferencePayload {
    value: String,
}

impl BtreesetSymmetricDifferencePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-symmetric-difference:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-symmetric-difference:{}", self.value)
    }
}

fn btreeset_symmetric_difference_left(raw: &str) -> BTreeSet<BtreesetSymmetricDifferencePayload> {
    let mut payloads = BTreeSet::new();
    payloads.insert(BtreesetSymmetricDifferencePayload::new(raw));
    payloads.insert(BtreesetSymmetricDifferencePayload::new("shared"));
    payloads
}

fn btreeset_symmetric_difference_right() -> BTreeSet<BtreesetSymmetricDifferencePayload> {
    let mut payloads = BTreeSet::new();
    payloads.insert(BtreesetSymmetricDifferencePayload::new("shared"));
    payloads.insert(BtreesetSymmetricDifferencePayload::new("tail"));
    payloads
}

pub fn selected_btreeset_symmetric_difference(raw: &str) -> String {
    let left = btreeset_symmetric_difference_left(raw);
    let right = btreeset_symmetric_difference_right();
    left.symmetric_difference(&right)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "btreeset-symmetric-difference:missing".to_string())
}

pub fn dead_live_btreeset_symmetric_difference(raw: &str) -> String {
    BtreesetSymmetricDifferencePayload::new(raw).dead_method()
}
