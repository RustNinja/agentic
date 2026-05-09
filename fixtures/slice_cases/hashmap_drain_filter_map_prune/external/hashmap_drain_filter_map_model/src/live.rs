use std::collections::HashMap;
pub struct HashmapDrainFilterMapPayload {
    value: String,
}

impl HashmapDrainFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-drain-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashmap-drain-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-drain-filter-map:{}", self.value)
    }
}

pub fn selected_hashmap_drain_filter_map(raw: &str) -> String {
    let mut values = HashMap::new();
    values.insert(raw.to_string(), HashmapDrainFilterMapPayload::new(raw));
    let rendered = values
        .drain()
        .filter_map(|(_, payload)| Some(payload.render_label()))
        .next()
        .unwrap_or_else(|| "missing".to_string());
    rendered
}

pub fn dead_live_hashmap_drain_filter_map(raw: &str) -> String {
    HashmapDrainFilterMapPayload::new(raw).unused_label()
}
