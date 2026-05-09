use std::collections::HashMap;
pub struct HashmapRemoveEntryMapPayload {
    value: String,
}

impl HashmapRemoveEntryMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-remove-entry-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashmap-remove-entry-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-remove-entry-map:{}", self.value)
    }
}

pub fn selected_hashmap_remove_entry_map(raw: &str) -> String {
    let mut values = HashMap::new();
    values.insert(raw.to_string(), HashmapRemoveEntryMapPayload::new(raw));
    values
        .remove_entry(raw)
        .map(|(_, payload)| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_hashmap_remove_entry_map(raw: &str) -> String {
    HashmapRemoveEntryMapPayload::new(raw).unused_label()
}
