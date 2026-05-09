use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapEntryOrDefaultKey {
    value: &'static str,
}

impl BtreemapEntryOrDefaultKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct BtreemapEntryOrDefaultPayload {
    value: String,
    touched: bool,
}

impl BtreemapEntryOrDefaultPayload {
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
        format!("btreemap-entry-or-default:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-or-default:{}", self.value)
    }
}

impl Default for BtreemapEntryOrDefaultPayload {
    fn default() -> Self {
        Self::new("default")
    }
}

fn btreemap_entry_or_default_entries(
) -> BTreeMap<BtreemapEntryOrDefaultKey, BtreemapEntryOrDefaultPayload> {
    BTreeMap::new()
}

pub fn selected_btreemap_entry_or_default(_raw: &str) -> String {
    let mut entries = btreemap_entry_or_default_entries();
    entries
        .entry(BtreemapEntryOrDefaultKey::live())
        .or_default()
        .render_label()
}

pub fn dead_live_btreemap_entry_or_default(raw: &str) -> String {
    BtreemapEntryOrDefaultPayload::new(raw).dead_method()
}
