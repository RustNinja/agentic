use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapValuesLastKey {
    value: &'static str,
}

impl BtreemapValuesLastKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct BtreemapValuesLastPayload {
    value: String,
    touched: bool,
}

impl BtreemapValuesLastPayload {
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
        format!("btreemap-values-last:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-values-last:{}", self.value)
    }
}

fn btreemap_values_last_entries(raw: &str) -> BTreeMap<BtreemapValuesLastKey, BtreemapValuesLastPayload> {
    let mut entries = BTreeMap::new();
    entries.insert(BtreemapValuesLastKey::live(), BtreemapValuesLastPayload::new(raw));
    entries.insert(BtreemapValuesLastKey::tail(), BtreemapValuesLastPayload::new("tail"));
    entries
}

pub fn selected_btreemap_values_last(raw: &str) -> String {
    let entries = btreemap_values_last_entries(raw);
    entries
        .values()
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "btreemap-values-last:missing".to_string())
}

pub fn dead_live_btreemap_values_last(raw: &str) -> String {
    BtreemapValuesLastPayload::new(raw).dead_method()
}
