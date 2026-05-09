use std::collections::BTreeMap;

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct BtreemapEntryAndModifyKey {
    value: &'static str,
}

impl BtreemapEntryAndModifyKey {
    pub fn live() -> Self {
        Self { value: "live" }
    }
}

pub struct BtreemapEntryAndModifyPayload {
    value: String,
    touched: bool,
}

impl BtreemapEntryAndModifyPayload {
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
        format!("btreemap-entry-and-modify:{}{}", self.value, suffix)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-entry-and-modify:{}", self.value)
    }
}

fn btreemap_entry_and_modify_entries(
) -> BTreeMap<BtreemapEntryAndModifyKey, BtreemapEntryAndModifyPayload> {
    BTreeMap::new()
}

pub fn selected_btreemap_entry_and_modify(raw: &str) -> String {
    let mut entries = btreemap_entry_and_modify_entries();
    entries.insert(
        BtreemapEntryAndModifyKey::live(),
        BtreemapEntryAndModifyPayload::new(raw),
    );
    entries
        .entry(BtreemapEntryAndModifyKey::live())
        .and_modify(|payload| {
            payload.mark_live();
        })
        .or_insert(BtreemapEntryAndModifyPayload::new("fallback"))
        .render_label()
}

pub fn dead_live_btreemap_entry_and_modify(raw: &str) -> String {
    BtreemapEntryAndModifyPayload::new(raw).dead_method()
}
