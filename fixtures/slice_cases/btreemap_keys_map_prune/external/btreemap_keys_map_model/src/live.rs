use std::collections::BTreeMap;
pub struct BtreemapKeysMapPayload {
    value: String,
}

impl BtreemapKeysMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-keys-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-keys-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-keys-map:{}", self.value)
    }
}

pub fn selected_btreemap_keys_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert(raw.to_string(), 1usize);
    values
        .keys()
        .map(|key| BtreemapKeysMapPayload::new(key).render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_keys_map(raw: &str) -> String {
    BtreemapKeysMapPayload::new(raw).unused_label()
}
