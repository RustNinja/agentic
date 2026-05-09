use std::collections::HashMap;
pub struct HashmapValuesFilterMapPayload {
    value: String,
}

impl HashmapValuesFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-values-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashmap-values-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-values-filter-map:{}", self.value)
    }
}

pub fn selected_hashmap_values_filter_map(raw: &str) -> String {
    let mut values = HashMap::new();
    values.insert("live", HashmapValuesFilterMapPayload::new(raw));
    values.insert("empty", HashmapValuesFilterMapPayload::new(""));
    values
        .values()
        .filter_map(|payload| (!payload.value.is_empty()).then(|| payload.render_label()))
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_hashmap_values_filter_map(raw: &str) -> String {
    HashmapValuesFilterMapPayload::new(raw).unused_label()
}
