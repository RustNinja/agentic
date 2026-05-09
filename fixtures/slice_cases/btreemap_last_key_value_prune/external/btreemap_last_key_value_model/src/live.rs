use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapLastKeyValueKey {
    value: String,
}

impl BtreemapLastKeyValueKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-last-key-value-key:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-btreemap-last-key-value-key:{}", self.value)
    }
}

#[derive(Clone)]
pub struct BtreemapLastKeyValuePayload {
    value: String,
}

impl BtreemapLastKeyValuePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-last-key-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-last-key-value:{}", self.value)
    }
}

fn btreemap_last_key_value_map(
    raw: &str,
) -> BTreeMap<BtreemapLastKeyValueKey, BtreemapLastKeyValuePayload> {
    let mut map = BTreeMap::new();
    map.insert(
        BtreemapLastKeyValueKey::new(raw),
        BtreemapLastKeyValuePayload::new(raw),
    );
    map.insert(
        BtreemapLastKeyValueKey::new("zz"),
        BtreemapLastKeyValuePayload::new("tail"),
    );
    map
}

pub fn selected_btreemap_last_key_value(raw: &str) -> String {
    let map = btreemap_last_key_value_map(raw);
    map.last_key_value()
        .map(|(key, payload)| format!("{}={}", key.render_key(), payload.render_label()))
        .unwrap_or_else(|| "btreemap-last-key-value:missing".to_string())
}

pub fn dead_live_btreemap_last_key_value(raw: &str) -> String {
    let key = BtreemapLastKeyValueKey::new(raw);
    let payload = BtreemapLastKeyValuePayload::new(raw);
    format!("{}={}", key.dead_key_method(), payload.dead_method())
}
