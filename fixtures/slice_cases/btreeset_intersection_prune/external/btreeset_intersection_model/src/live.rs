use std::collections::BTreeSet;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreesetIntersectionPayload {
    value: String,
}

impl BtreesetIntersectionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-intersection:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-intersection:{}", self.value)
    }
}

fn btreeset_intersection_left(raw: &str) -> BTreeSet<BtreesetIntersectionPayload> {
    let mut payloads = BTreeSet::new();
    payloads.insert(BtreesetIntersectionPayload::new(raw));
    payloads.insert(BtreesetIntersectionPayload::new("shared"));
    payloads
}

fn btreeset_intersection_right() -> BTreeSet<BtreesetIntersectionPayload> {
    let mut payloads = BTreeSet::new();
    payloads.insert(BtreesetIntersectionPayload::new("shared"));
    payloads.insert(BtreesetIntersectionPayload::new("tail"));
    payloads
}

pub fn selected_btreeset_intersection(raw: &str) -> String {
    let left = btreeset_intersection_left(raw);
    let right = btreeset_intersection_right();
    left.intersection(&right)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "btreeset-intersection:missing".to_string())
}

pub fn dead_live_btreeset_intersection(raw: &str) -> String {
    BtreesetIntersectionPayload::new(raw).dead_method()
}
