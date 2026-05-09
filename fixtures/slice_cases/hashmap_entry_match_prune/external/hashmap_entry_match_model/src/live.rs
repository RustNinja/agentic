use std::collections::HashMap;

pub struct HashmapEntryMatchPayload {
    value: String,
}

impl HashmapEntryMatchPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-entry-match:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-entry-match:{}", self.value)
    }
}

pub fn selected_hashmap_entry_match(raw: &str) -> String {
    let mut map = HashMap::new();
    map.insert("live".to_string(), HashmapEntryMatchPayload::new(raw));
    match map.entry("live".to_string()) {
        std::collections::hash_map::Entry::Occupied(entry) => entry.get().render_label(),
        std::collections::hash_map::Entry::Vacant(entry) => entry
            .insert(HashmapEntryMatchPayload::new("fallback"))
            .render_label(),
    }
}

pub fn dead_live_hashmap_entry_match(raw: &str) -> String {
    HashmapEntryMatchPayload::new(raw).unused_label()
}
