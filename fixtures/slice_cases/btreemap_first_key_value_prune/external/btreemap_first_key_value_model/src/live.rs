use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapFirstKeyValueKey {
    value: String,
}

impl BtreemapFirstKeyValueKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-first-key-value-key:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-btreemap-first-key-value-key:{}", self.value)
    }
}

#[derive(Clone)]
pub struct BtreemapFirstKeyValuePayload {
    value: String,
}

impl BtreemapFirstKeyValuePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-first-key-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-first-key-value:{}", self.value)
    }
}

fn btreemap_first_key_value_map(
    raw: &str,
) -> BTreeMap<BtreemapFirstKeyValueKey, BtreemapFirstKeyValuePayload> {
    let mut map = BTreeMap::new();
    map.insert(
        BtreemapFirstKeyValueKey::new(raw),
        BtreemapFirstKeyValuePayload::new(raw),
    );
    map.insert(
        BtreemapFirstKeyValueKey::new("zz"),
        BtreemapFirstKeyValuePayload::new("tail"),
    );
    map
}

pub fn selected_btreemap_first_key_value(raw: &str) -> String {
    let map = btreemap_first_key_value_map(raw);
    map.first_key_value()
        .map(|(key, payload)| format!("{}={}", key.render_key(), payload.render_label()))
        .unwrap_or_else(|| "btreemap-first-key-value:missing".to_string())
}

pub fn dead_live_btreemap_first_key_value(raw: &str) -> String {
    let key = BtreemapFirstKeyValueKey::new(raw);
    let payload = BtreemapFirstKeyValuePayload::new(raw);
    format!("{}={}", key.dead_key_method(), payload.dead_method())
}
