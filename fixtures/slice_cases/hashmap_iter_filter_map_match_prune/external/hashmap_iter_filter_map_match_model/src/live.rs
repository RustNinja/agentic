use std::collections::HashMap;

pub struct HashmapIterFilterMapMatchPayload {
    value: String,
}

impl HashmapIterFilterMapMatchPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-iter-filter-map-match:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-iter-filter-map-match:{}", self.value)
    }
}

pub fn selected_hashmap_iter_filter_map_match(raw: &str) -> String {
    let mut entries: HashMap<String, HashmapIterFilterMapMatchPayload> = HashMap::new();
    entries.insert(
        "live".to_string(),
        HashmapIterFilterMapMatchPayload::new(raw),
    );
    entries.insert(
        "dead".to_string(),
        HashmapIterFilterMapMatchPayload::new(""),
    );
    entries
        .iter()
        .filter_map(|(key, payload)| match key.as_str() {
            "live" => Some(payload.render_label()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_hashmap_iter_filter_map_match(raw: &str) -> String {
    HashmapIterFilterMapMatchPayload::new(raw).unused_label()
}
