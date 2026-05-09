use std::collections::BTreeMap;
pub struct BtreemapFirstEntryMutMapPayload {
    value: String,
}

impl BtreemapFirstEntryMutMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-first-entry-mut-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-first-entry-mut-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-first-entry-mut-map:{}", self.value)
    }
}

pub fn selected_btreemap_first_entry_mut_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert("live", BtreemapFirstEntryMutMapPayload::new(raw));
    values
        .first_entry()
        .map(|mut entry| entry.get_mut().bump_and_render())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_first_entry_mut_map(raw: &str) -> String {
    BtreemapFirstEntryMutMapPayload::new(raw).unused_label()
}
