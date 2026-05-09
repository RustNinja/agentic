use std::collections::{btree_map::Entry, BTreeMap};
pub struct BtreemapEntryRemoveMapPayload {
    value: String,
}

impl BtreemapEntryRemoveMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-entry-remove-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-entry-remove-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-entry-remove-map:{}", self.value)
    }
}

pub fn selected_btreemap_entry_remove_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert(raw.to_string(), BtreemapEntryRemoveMapPayload::new(raw));
    match values.entry(raw.to_string()) {
        Entry::Occupied(entry) => {
            let (_, payload) = entry.remove_entry();
            payload.render_label()
        }
        Entry::Vacant(entry) => entry
            .insert(BtreemapEntryRemoveMapPayload::new(raw))
            .render_label(),
    }
}

pub fn dead_live_btreemap_entry_remove_map(raw: &str) -> String {
    BtreemapEntryRemoveMapPayload::new(raw).unused_label()
}
