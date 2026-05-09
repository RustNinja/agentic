use std::collections::BTreeMap;
pub struct BtreemapRemoveEntryMapPayload {
    value: String,
}

impl BtreemapRemoveEntryMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-remove-entry-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-remove-entry-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-remove-entry-map:{}", self.value)
    }
}

pub fn selected_btreemap_remove_entry_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert(raw.to_string(), BtreemapRemoveEntryMapPayload::new(raw));
    values
        .remove_entry(raw)
        .map(|(_, payload)| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_remove_entry_map(raw: &str) -> String {
    BtreemapRemoveEntryMapPayload::new(raw).unused_label()
}
