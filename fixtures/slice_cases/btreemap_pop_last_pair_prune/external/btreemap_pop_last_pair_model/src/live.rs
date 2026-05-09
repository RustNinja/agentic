use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapPopLastPairKey {
    value: String,
}

impl BtreemapPopLastPairKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-pop-last-pair-key:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-btreemap-pop-last-pair-key:{}", self.value)
    }
}

#[derive(Clone)]
pub struct BtreemapPopLastPairPayload {
    value: String,
}

impl BtreemapPopLastPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-pop-last-pair:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-pop-last-pair:{}", self.value)
    }
}

fn btreemap_pop_last_pair_map(
    raw: &str,
) -> BTreeMap<BtreemapPopLastPairKey, BtreemapPopLastPairPayload> {
    let mut map = BTreeMap::new();
    map.insert(
        BtreemapPopLastPairKey::new(raw),
        BtreemapPopLastPairPayload::new(raw),
    );
    map.insert(
        BtreemapPopLastPairKey::new("zz"),
        BtreemapPopLastPairPayload::new("tail"),
    );
    map
}

pub fn selected_btreemap_pop_last_pair(raw: &str) -> String {
    let mut map = btreemap_pop_last_pair_map(raw);
    map.pop_last()
        .map(|(key, payload)| format!("{}={}", key.render_key(), payload.render_label()))
        .unwrap_or_else(|| "btreemap-pop-last-pair:missing".to_string())
}

pub fn dead_live_btreemap_pop_last_pair(raw: &str) -> String {
    let key = BtreemapPopLastPairKey::new(raw);
    let payload = BtreemapPopLastPairPayload::new(raw);
    format!("{}={}", key.dead_key_method(), payload.dead_method())
}
