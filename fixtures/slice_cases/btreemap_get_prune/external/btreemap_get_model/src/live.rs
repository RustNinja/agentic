use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapGetKey {
    value: &'static str,
}

impl BtreemapGetKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct BtreemapGetPayload {
    value: String,
    touched: bool,
}

impl BtreemapGetPayload {
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
        format!("btreemap-get:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-get:{}", self.value)
    }
}

fn btreemap_get_entries(raw: &str) -> BTreeMap<BtreemapGetKey, BtreemapGetPayload> {
    let mut entries = BTreeMap::new();
    entries.insert(BtreemapGetKey::live(), BtreemapGetPayload::new(raw));
    entries.insert(BtreemapGetKey::tail(), BtreemapGetPayload::new("tail"));
    entries
}

pub fn selected_btreemap_get(raw: &str) -> String {
    let entries = btreemap_get_entries(raw);
    entries
        .get(&BtreemapGetKey::live())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "btreemap-get:missing".to_string())
}

pub fn dead_live_btreemap_get(raw: &str) -> String {
    BtreemapGetPayload::new(raw).dead_method()
}
