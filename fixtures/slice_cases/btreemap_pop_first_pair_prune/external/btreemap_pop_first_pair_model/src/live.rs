use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapPopFirstPairKey {
    value: String,
}

impl BtreemapPopFirstPairKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-pop-first-pair-key:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-btreemap-pop-first-pair-key:{}", self.value)
    }
}

#[derive(Clone)]
pub struct BtreemapPopFirstPairPayload {
    value: String,
}

impl BtreemapPopFirstPairPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-pop-first-pair:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-pop-first-pair:{}", self.value)
    }
}

fn btreemap_pop_first_pair_map(
    raw: &str,
) -> BTreeMap<BtreemapPopFirstPairKey, BtreemapPopFirstPairPayload> {
    let mut map = BTreeMap::new();
    map.insert(
        BtreemapPopFirstPairKey::new(raw),
        BtreemapPopFirstPairPayload::new(raw),
    );
    map.insert(
        BtreemapPopFirstPairKey::new("zz"),
        BtreemapPopFirstPairPayload::new("tail"),
    );
    map
}

pub fn selected_btreemap_pop_first_pair(raw: &str) -> String {
    let mut map = btreemap_pop_first_pair_map(raw);
    map.pop_first()
        .map(|(key, payload)| format!("{}={}", key.render_key(), payload.render_label()))
        .unwrap_or_else(|| "btreemap-pop-first-pair:missing".to_string())
}

pub fn dead_live_btreemap_pop_first_pair(raw: &str) -> String {
    let key = BtreemapPopFirstPairKey::new(raw);
    let payload = BtreemapPopFirstPairPayload::new(raw);
    format!("{}={}", key.dead_key_method(), payload.dead_method())
}
