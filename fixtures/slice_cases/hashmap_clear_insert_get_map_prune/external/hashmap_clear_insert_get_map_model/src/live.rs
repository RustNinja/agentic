use std::collections::HashMap;
pub struct HashmapClearInsertGetMapPayload {
    value: String,
}

impl HashmapClearInsertGetMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-clear-insert-get-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashmap-clear-insert-get-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-clear-insert-get-map:{}", self.value)
    }
}

pub fn selected_hashmap_clear_insert_get_map(raw: &str) -> String {
    let mut values = HashMap::new();
    values.insert("dead".to_string(), HashmapClearInsertGetMapPayload::new("dead"));
    values.clear();
    values.insert(raw.to_string(), HashmapClearInsertGetMapPayload::new(raw));
    values
        .get(raw)
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_hashmap_clear_insert_get_map(raw: &str) -> String {
    HashmapClearInsertGetMapPayload::new(raw).unused_label()
}
