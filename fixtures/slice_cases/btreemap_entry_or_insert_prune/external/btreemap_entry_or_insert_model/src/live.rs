use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapEntryOrInsertKey {
    value: &'static str,
}

impl BtreemapEntryOrInsertKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct BtreemapEntryOrInsertPayload {
    value: String,
    touched: bool,
}

impl BtreemapEntryOrInsertPayload {
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
        format!("btreemap-entry-or-insert:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-or-insert:{}", self.value)
    }
}

fn btreemap_entry_or_insert_entries(
) -> BTreeMap<BtreemapEntryOrInsertKey, BtreemapEntryOrInsertPayload> {
    BTreeMap::new()
}

pub fn selected_btreemap_entry_or_insert(raw: &str) -> String {
    let mut entries = btreemap_entry_or_insert_entries();
    entries
        .entry(BtreemapEntryOrInsertKey::live())
        .or_insert(BtreemapEntryOrInsertPayload::new(raw))
        .render_label()
}

pub fn dead_live_btreemap_entry_or_insert(raw: &str) -> String {
    BtreemapEntryOrInsertPayload::new(raw).dead_method()
}
