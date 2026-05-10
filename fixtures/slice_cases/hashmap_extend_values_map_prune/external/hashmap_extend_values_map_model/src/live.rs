use std::collections::HashMap;
pub struct HashmapExtendValuesMapPayload {
    value: String,
}

impl HashmapExtendValuesMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-extend-values-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashmap-extend-values-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-extend-values-map:{}", self.value)
    }
}

pub fn selected_hashmap_extend_values_map(raw: &str) -> String {
    let mut values = HashMap::new();
    values.extend([(raw.to_string(), HashmapExtendValuesMapPayload::new(raw))]);
    values
        .values()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_hashmap_extend_values_map(raw: &str) -> String {
    HashmapExtendValuesMapPayload::new(raw).unused_label()
}
