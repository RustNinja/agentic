use std::collections::BTreeMap;
pub struct BtreemapSplitOffKeysMapPayload {
    value: String,
}

impl BtreemapSplitOffKeysMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-split-off-keys-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-split-off-keys-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-split-off-keys-map:{}", self.value)
    }
}

pub fn selected_btreemap_split_off_keys_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert(raw.to_string(), 1usize);
    let right = values.split_off(raw);
    right
        .keys()
        .map(|key| BtreemapSplitOffKeysMapPayload::new(key).render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_split_off_keys_map(raw: &str) -> String {
    BtreemapSplitOffKeysMapPayload::new(raw).unused_label()
}
