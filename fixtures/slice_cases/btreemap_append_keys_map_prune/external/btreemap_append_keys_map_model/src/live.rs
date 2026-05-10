use std::collections::BTreeMap;
pub struct BtreemapAppendKeysMapPayload {
    value: String,
}

impl BtreemapAppendKeysMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-append-keys-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-append-keys-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-append-keys-map:{}", self.value)
    }
}

pub fn selected_btreemap_append_keys_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    let mut extras = BTreeMap::new();
    extras.insert(raw.to_string(), BtreemapAppendKeysMapPayload::new(raw));
    values.append(&mut extras);
    values
        .keys()
        .map(|key| BtreemapAppendKeysMapPayload::new(key).render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_append_keys_map(raw: &str) -> String {
    BtreemapAppendKeysMapPayload::new(raw).unused_label()
}
