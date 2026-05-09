use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapAppendValuesKey {
    value: String,
}

impl BtreemapAppendValuesKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-append-values-key:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-btreemap-append-values-key:{}", self.value)
    }
}

#[derive(Clone)]
pub struct BtreemapAppendValuesPayload {
    value: String,
}

impl BtreemapAppendValuesPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-append-values:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-append-values:{}", self.value)
    }
}

fn btreemap_append_values_map(
    raw: &str,
) -> BTreeMap<BtreemapAppendValuesKey, BtreemapAppendValuesPayload> {
    let mut payloads = BTreeMap::new();
    payloads.insert(
        BtreemapAppendValuesKey::new(raw),
        BtreemapAppendValuesPayload::new(raw),
    );
    payloads.insert(
        BtreemapAppendValuesKey::new("tail"),
        BtreemapAppendValuesPayload::new("tail"),
    );
    payloads
}

pub fn selected_btreemap_append_values(raw: &str) -> String {
    let mut payloads = btreemap_append_values_map(raw);
    let mut extras = btreemap_append_values_map("tail");
    payloads.append(&mut extras);
    payloads
        .values()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "btreemap-append-values:missing".to_string())
}

pub fn dead_live_btreemap_append_values(raw: &str) -> String {
    let key = BtreemapAppendValuesKey::new(raw);
    let payload = BtreemapAppendValuesPayload::new(raw);
    format!("{}={}", key.dead_key_method(), payload.dead_method())
}
