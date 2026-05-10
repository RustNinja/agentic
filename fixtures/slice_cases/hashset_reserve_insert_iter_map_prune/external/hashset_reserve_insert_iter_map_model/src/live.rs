use std::collections::HashSet;
pub struct HashsetReserveInsertIterMapPayload {
    value: String,
}

impl HashsetReserveInsertIterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("hashset-reserve-insert-iter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("hashset-reserve-insert-iter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-hashset-reserve-insert-iter-map:{}", self.value)
    }
}

pub fn selected_hashset_reserve_insert_iter_map(raw: &str) -> String {
    let mut values = HashSet::new();
    values.reserve(1);
    values.insert(raw.to_string());
    values
        .iter()
        .map(|value| HashsetReserveInsertIterMapPayload::new(value).render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_hashset_reserve_insert_iter_map(raw: &str) -> String {
    HashsetReserveInsertIterMapPayload::new(raw).unused_label()
}
