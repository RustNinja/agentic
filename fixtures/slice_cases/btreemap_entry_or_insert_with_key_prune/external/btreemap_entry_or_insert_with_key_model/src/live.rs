use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapEntryOrInsertWithKeyKey {
    value: &'static str,
}

impl BtreemapEntryOrInsertWithKeyKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct BtreemapEntryOrInsertWithKeyPayload {
    value: String,
    touched: bool,
}

impl BtreemapEntryOrInsertWithKeyPayload {
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
        format!("btreemap-entry-or-insert-with-key:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-or-insert-with-key:{}", self.value)
    }
}

fn btreemap_entry_or_insert_with_key_entries(
) -> BTreeMap<BtreemapEntryOrInsertWithKeyKey, BtreemapEntryOrInsertWithKeyPayload> {
    BTreeMap::new()
}

pub fn selected_btreemap_entry_or_insert_with_key(raw: &str) -> String {
    let mut entries = btreemap_entry_or_insert_with_key_entries();
    entries
        .entry(BtreemapEntryOrInsertWithKeyKey::live())
        .or_insert_with_key(|_| BtreemapEntryOrInsertWithKeyPayload::new(raw))
        .render_label()
}

pub fn dead_live_btreemap_entry_or_insert_with_key(raw: &str) -> String {
    BtreemapEntryOrInsertWithKeyPayload::new(raw).dead_method()
}
