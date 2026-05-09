use std::collections::BTreeSet;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreesetUnionPayload {
    value: String,
}

impl BtreesetUnionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-union:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreeset-union:{}", self.value)
    }
}

fn btreeset_union_left(raw: &str) -> BTreeSet<BtreesetUnionPayload> {
    let mut payloads = BTreeSet::new();
    payloads.insert(BtreesetUnionPayload::new(raw));
    payloads.insert(BtreesetUnionPayload::new("shared"));
    payloads
}

fn btreeset_union_right() -> BTreeSet<BtreesetUnionPayload> {
    let mut payloads = BTreeSet::new();
    payloads.insert(BtreesetUnionPayload::new("shared"));
    payloads.insert(BtreesetUnionPayload::new("tail"));
    payloads
}

pub fn selected_btreeset_union(raw: &str) -> String {
    let left = btreeset_union_left(raw);
    let right = btreeset_union_right();
    left.union(&right)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "btreeset-union:missing".to_string())
}

pub fn dead_live_btreeset_union(raw: &str) -> String {
    BtreesetUnionPayload::new(raw).dead_method()
}
