use std::collections::BTreeMap;

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapValuesMutKey {
    value: String,
}

impl BtreemapValuesMutKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn dead_key_method(&self) -> String {
        format!("dead-btreemap-values-mut-key:{}", self.value)
    }
}

#[derive(Clone)]
pub struct BtreemapValuesMutPayload {
    value: String,
}

impl BtreemapValuesMutPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-values-mut:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("btreemap-values-mut:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-values-mut:{}", self.value)
    }
}

fn btreemap_values_mut_map(raw: &str) -> BTreeMap<BtreemapValuesMutKey, BtreemapValuesMutPayload> {
    let mut payloads = BTreeMap::new();
    payloads.insert(
        BtreemapValuesMutKey::new(raw),
        BtreemapValuesMutPayload::new(raw),
    );
    payloads.insert(
        BtreemapValuesMutKey::new("tail"),
        BtreemapValuesMutPayload::new("tail"),
    );
    payloads
}

pub fn selected_btreemap_values_mut(raw: &str) -> String {
    let mut payloads = btreemap_values_mut_map(raw);
    payloads
        .values_mut()
        .map(|payload| payload.bump_and_render())
        .next()
        .unwrap_or_else(|| "btreemap-values-mut:missing".to_string())
}

pub fn dead_live_btreemap_values_mut(raw: &str) -> String {
    let key = BtreemapValuesMutKey::new(raw);
    let payload = BtreemapValuesMutPayload::new(raw);
    format!("{}={}", key.dead_key_method(), payload.dead_method())
}
