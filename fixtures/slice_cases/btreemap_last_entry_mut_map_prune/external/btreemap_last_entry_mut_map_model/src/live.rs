use std::collections::BTreeMap;
pub struct BtreemapLastEntryMutMapPayload {
    value: String,
}

impl BtreemapLastEntryMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-last-entry-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-last-entry-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-last-entry-mut-map:{}", self.value)
    }
}

pub fn selected_btreemap_last_entry_mut_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert("live", BtreemapLastEntryMutMapPayload::new(raw));
    values
        .last_entry()
        .map(|mut entry| entry.get_mut().bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_last_entry_mut_map(raw: &str) -> String {
    BtreemapLastEntryMutMapPayload::new(raw).unused_label()
}
