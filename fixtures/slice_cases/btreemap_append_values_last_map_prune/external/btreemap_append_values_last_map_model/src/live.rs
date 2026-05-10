use std::collections::BTreeMap;
pub struct BtreemapAppendValuesLastMapPayload {
    value: String,
}

impl BtreemapAppendValuesLastMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-append-values-last-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-append-values-last-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-append-values-last-map:{}", self.value)
    }
}

pub fn selected_btreemap_append_values_last_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert(1usize, BtreemapAppendValuesLastMapPayload::new("old"));
    let mut extras = BTreeMap::new();
    extras.insert(2usize, BtreemapAppendValuesLastMapPayload::new(raw));
    values.append(&mut extras);
    values
        .values()
        .last()
        .map(|payload| payload.render_label())
        .unwrap_or_default()
}

pub fn dead_live_btreemap_append_values_last_map(raw: &str) -> String {
    BtreemapAppendValuesLastMapPayload::new(raw).unused_label()
}
