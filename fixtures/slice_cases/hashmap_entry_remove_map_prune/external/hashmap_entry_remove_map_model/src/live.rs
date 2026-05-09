use std::collections::{hash_map::Entry, HashMap};
pub struct HashmapEntryRemoveMapPayload {
    value: String,
}

impl HashmapEntryRemoveMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-entry-remove-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashmap-entry-remove-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-entry-remove-map:{}", self.value)
    }
}

pub fn selected_hashmap_entry_remove_map(raw: &str) -> String {
    let mut values = HashMap::new();
    values.insert(raw.to_string(), HashmapEntryRemoveMapPayload::new(raw));
    match values.entry(raw.to_string()) {
        Entry::Occupied(entry) => {
            let (_, payload) = entry.remove_entry();
            payload.render_label()
        }
        Entry::Vacant(entry) => entry
            .insert(HashmapEntryRemoveMapPayload::new(raw))
            .render_label(),
    }
}

pub fn dead_live_hashmap_entry_remove_map(raw: &str) -> String {
    HashmapEntryRemoveMapPayload::new(raw).unused_label()
}
