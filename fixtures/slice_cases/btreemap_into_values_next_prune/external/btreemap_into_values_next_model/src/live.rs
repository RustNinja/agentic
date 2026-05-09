use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct BtreemapIntoValuesNextKey {
    value: &'static str,
}

impl BtreemapIntoValuesNextKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }

    pub fn tail() -> Self {
        Self { value: "tail" }
    }
}

pub struct BtreemapIntoValuesNextPayload {
    value: String,
    touched: bool,
}

impl BtreemapIntoValuesNextPayload {
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
        format!("btreemap-into-values-next:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-into-values-next:{}", self.value)
    }
}

fn btreemap_into_values_next_entries(raw: &str) -> BTreeMap<BtreemapIntoValuesNextKey, BtreemapIntoValuesNextPayload> {
    let mut entries = BTreeMap::new();
    entries.insert(BtreemapIntoValuesNextKey::live(), BtreemapIntoValuesNextPayload::new(raw));
    entries.insert(BtreemapIntoValuesNextKey::tail(), BtreemapIntoValuesNextPayload::new("tail"));
    entries
}

pub fn selected_btreemap_into_values_next(raw: &str) -> String {
    let entries = btreemap_into_values_next_entries(raw);
    entries
        .into_values()
        .next()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "btreemap-into-values-next:missing".to_string())
}

pub fn dead_live_btreemap_into_values_next(raw: &str) -> String {
    BtreemapIntoValuesNextPayload::new(raw).dead_method()
}
