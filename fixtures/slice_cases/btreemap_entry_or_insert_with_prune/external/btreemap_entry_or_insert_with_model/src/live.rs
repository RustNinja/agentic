use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapEntryOrInsertWithKey {
    value: &'static str,
}

impl BtreemapEntryOrInsertWithKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct BtreemapEntryOrInsertWithPayload {
    value: String,
    touched: bool,
}

impl BtreemapEntryOrInsertWithPayload {
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
        format!("btreemap-entry-or-insert-with:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-or-insert-with:{}", self.value)
    }
}

fn btreemap_entry_or_insert_with_entries(
) -> BTreeMap<BtreemapEntryOrInsertWithKey, BtreemapEntryOrInsertWithPayload> {
    BTreeMap::new()
}

pub fn selected_btreemap_entry_or_insert_with(raw: &str) -> String {
    let mut entries = btreemap_entry_or_insert_with_entries();
    entries
        .entry(BtreemapEntryOrInsertWithKey::live())
        .or_insert_with(|| BtreemapEntryOrInsertWithPayload::new(raw))
        .render_label()
}

pub fn dead_live_btreemap_entry_or_insert_with(raw: &str) -> String {
    BtreemapEntryOrInsertWithPayload::new(raw).dead_method()
}
