use std::collections::HashMap;
pub struct HashmapReserveInsertValuesMapPayload {
    value: String,
}

impl HashmapReserveInsertValuesMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashmap-reserve-insert-values-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashmap-reserve-insert-values-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashmap-reserve-insert-values-map:{}", self.value)
    }
}

pub fn selected_hashmap_reserve_insert_values_map(raw: &str) -> String {
    let mut values = HashMap::new();
    values.reserve(1);
    values.insert(
        raw.to_string(),
        HashmapReserveInsertValuesMapPayload::new(raw),
    );
    values
        .values()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_hashmap_reserve_insert_values_map(raw: &str) -> String {
    HashmapReserveInsertValuesMapPayload::new(raw).unused_label()
}
