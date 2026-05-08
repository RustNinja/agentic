pub struct TupleFindMapKey {
    value: String,
}

impl TupleFindMapKey {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_render(&self, value: &TupleFindMapValue) -> Option<String> {
        value
            .render_label()
            .contains(&self.value)
            .then(|| format!("tuple-find-map:{}:{}", self.value, value.render_label()))
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-find-map-key:{}", self.value)
    }
}

pub struct TupleFindMapValue {
    value: String,
}

impl TupleFindMapValue {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("find-value:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-tuple-find-map-value:{}", self.value)
    }
}

fn tuple_find_map_entries(raw: &str) -> Vec<(TupleFindMapKey, TupleFindMapValue)> {
    raw.split(',')
        .map(|part| (TupleFindMapKey::new(part), TupleFindMapValue::new(part)))
        .collect()
}

pub fn selected_tuple_find_map(raw: &str) -> String {
    let entries = tuple_find_map_entries(raw);
    entries
        .iter()
        .find_map(|(key, value)| key.maybe_render(value))
        .unwrap_or_else(|| "tuple-find-map:missing".to_string())
}

pub fn dead_live_tuple_find_map(raw: &str) -> String {
    TupleFindMapKey::new(raw).dead_method()
}
