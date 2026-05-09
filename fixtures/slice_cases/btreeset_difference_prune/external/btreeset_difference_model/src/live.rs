use std::collections::BTreeSet;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreesetDifferencePayload {
    value: String,
}

impl BtreesetDifferencePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-difference:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-difference:{}", self.value)
    }
}

fn btreeset_difference_left(raw: &str) -> BTreeSet<BtreesetDifferencePayload> {
    let mut payloads = BTreeSet::new();
    payloads.insert(BtreesetDifferencePayload::new(raw));
    payloads.insert(BtreesetDifferencePayload::new("shared"));
    payloads
}

fn btreeset_difference_right() -> BTreeSet<BtreesetDifferencePayload> {
    let mut payloads = BTreeSet::new();
    payloads.insert(BtreesetDifferencePayload::new("shared"));
    payloads.insert(BtreesetDifferencePayload::new("tail"));
    payloads
}

pub fn selected_btreeset_difference(raw: &str) -> String {
    let left = btreeset_difference_left(raw);
    let right = btreeset_difference_right();
    left.difference(&right)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "btreeset-difference:missing".to_string())
}

pub fn dead_live_btreeset_difference(raw: &str) -> String {
    BtreesetDifferencePayload::new(raw).dead_method()
}
