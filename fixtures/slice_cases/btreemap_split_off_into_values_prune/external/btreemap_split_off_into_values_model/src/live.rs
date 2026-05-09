use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapSplitOffIntoValuesKey {
    value: String,
}

impl BtreemapSplitOffIntoValuesKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_key(&self) -> String {
        format!("btreemap-split-off-into-values-key:{}", self.value)
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-btreemap-split-off-into-values-key:{}", self.value)
    }
}

#[derive(Clone)]
pub struct BtreemapSplitOffIntoValuesPayload {
    value: String,
}

impl BtreemapSplitOffIntoValuesPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-split-off-into-values:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-split-off-into-values:{}", self.value)
    }
}

fn btreemap_split_off_into_values_map(
    raw: &str,
) -> BTreeMap<BtreemapSplitOffIntoValuesKey, BtreemapSplitOffIntoValuesPayload> {
    let mut payloads = BTreeMap::new();
    payloads.insert(
        BtreemapSplitOffIntoValuesKey::new(raw),
        BtreemapSplitOffIntoValuesPayload::new(raw),
    );
    payloads.insert(
        BtreemapSplitOffIntoValuesKey::new("tail"),
        BtreemapSplitOffIntoValuesPayload::new("tail"),
    );
    payloads
}

pub fn selected_btreemap_split_off_into_values(raw: &str) -> String {
    let mut payloads = btreemap_split_off_into_values_map(raw);
    payloads
        .split_off(&BtreemapSplitOffIntoValuesKey::new("tail"))
        .into_values()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "btreemap-split-off-into-values:missing".to_string())
}

pub fn dead_live_btreemap_split_off_into_values(raw: &str) -> String {
    let key = BtreemapSplitOffIntoValuesKey::new(raw);
    let payload = BtreemapSplitOffIntoValuesPayload::new(raw);
    format!("{}={}", key.dead_key_method(), payload.dead_method())
}
