use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapRemoveKey {
    value: &'static str,
}

impl BtreemapRemoveKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct BtreemapRemovePayload {
    value: String,
    touched: bool,
}

impl BtreemapRemovePayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
            touched: false,
        }
    }

    pub fn mark_live(&mut self) -> &mut Self {
        self.touched = true;
        self
    }

    pub fn render_label(&self) -> String {
        let suffix = if self.touched { ":touched" } else { "" };
        format!("btreemap-remove:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-remove:{}", self.value)
    }
}

fn btreemap_remove_entries(raw: &str) -> BTreeMap<BtreemapRemoveKey, BtreemapRemovePayload> {
    let mut entries = BTreeMap::new();
    entries.insert(BtreemapRemoveKey::live(), BtreemapRemovePayload::new(raw));
    entries.insert(BtreemapRemoveKey::tail(), BtreemapRemovePayload::new("tail"));
    entries
}

pub fn selected_btreemap_remove(raw: &str) -> String {
    let mut entries = btreemap_remove_entries(raw);
    entries
        .remove(&BtreemapRemoveKey::live())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "btreemap-remove:missing".to_string())
}

pub fn dead_live_btreemap_remove(raw: &str) -> String {
    BtreemapRemovePayload::new(raw).dead_method()
}
