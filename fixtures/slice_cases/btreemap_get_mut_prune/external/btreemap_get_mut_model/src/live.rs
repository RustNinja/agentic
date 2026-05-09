use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapGetMutKey {
    value: &'static str,
}

impl BtreemapGetMutKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct BtreemapGetMutPayload {
    value: String,
    touched: bool,
}

impl BtreemapGetMutPayload {
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
        format!("btreemap-get-mut:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-get-mut:{}", self.value)
    }
}

fn btreemap_get_mut_entries(raw: &str) -> BTreeMap<BtreemapGetMutKey, BtreemapGetMutPayload> {
    let mut entries = BTreeMap::new();
    entries.insert(BtreemapGetMutKey::live(), BtreemapGetMutPayload::new(raw));
    entries.insert(BtreemapGetMutKey::tail(), BtreemapGetMutPayload::new("tail"));
    entries
}

pub fn selected_btreemap_get_mut(raw: &str) -> String {
    let mut entries = btreemap_get_mut_entries(raw);
    entries
        .get_mut(&BtreemapGetMutKey::live())
        .map(|payload| payload.mark_live().render_label())
        .unwrap_or_else(|| "btreemap-get-mut:missing".to_string())
}

pub fn dead_live_btreemap_get_mut(raw: &str) -> String {
    BtreemapGetMutPayload::new(raw).dead_method()
}
