use std::collections::HashMap;

pub struct HashmapGetKeyValueMapPayload {
    value: String,
}

impl HashmapGetKeyValueMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-get-key-value-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-get-key-value-map:{}", self.value)
    }
}

pub fn selected_hashmap_get_key_value_map(raw: &str) -> String {
    let mut map = HashMap::new();
    map.insert("live".to_string(), HashmapGetKeyValueMapPayload::new(raw));
    map.get_key_value("live")
        .map(|(key, payload)| format!("{key}:{}", payload.render_label()))
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_hashmap_get_key_value_map(raw: &str) -> String {
    HashmapGetKeyValueMapPayload::new(raw).unused_label()
}
