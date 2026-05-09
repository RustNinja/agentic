use std::collections::BTreeSet;
pub struct BtreesetLastMapPayload {
    value: String,
}

impl BtreesetLastMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreeset-last-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreeset-last-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreeset-last-map:{}", self.value)
    }
}

pub fn selected_btreeset_last_map(raw: &str) -> String {
    let mut values = BTreeSet::new();
    values.insert(raw.to_string());
    values
        .last()
        .map(|value| BtreesetLastMapPayload::new(value).render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreeset_last_map(raw: &str) -> String {
    BtreesetLastMapPayload::new(raw).unused_label()
}
