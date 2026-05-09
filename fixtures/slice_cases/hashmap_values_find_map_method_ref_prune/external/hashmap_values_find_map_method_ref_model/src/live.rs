use std::collections::HashMap;

pub struct HashmapValuesFindMapMethodRefPayload {
    value: String,
}

impl HashmapValuesFindMapMethodRefPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_label(&self) -> Option<String> {
        (!self.value.is_empty()).then(|| self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-values-find-map-method-ref:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-values-find-map-method-ref:{}", self.value)
    }
}

pub fn selected_hashmap_values_find_map_method_ref(raw: &str) -> String {
    let mut entries: HashMap<String, HashmapValuesFindMapMethodRefPayload> = HashMap::new();
    entries.insert(
        "empty".to_string(),
        HashmapValuesFindMapMethodRefPayload::new(""),
    );
    entries.insert(
        "live".to_string(),
        HashmapValuesFindMapMethodRefPayload::new(raw),
    );
    entries
        .values()
        .find_map(HashmapValuesFindMapMethodRefPayload::maybe_label)
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_hashmap_values_find_map_method_ref(raw: &str) -> String {
    HashmapValuesFindMapMethodRefPayload::new(raw).unused_label()
}
