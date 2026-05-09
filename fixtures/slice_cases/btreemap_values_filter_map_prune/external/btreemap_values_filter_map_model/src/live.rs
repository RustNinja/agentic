use std::collections::BTreeMap;
pub struct BtreemapValuesFilterMapPayload {
    value: String,
}

impl BtreemapValuesFilterMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-values-filter-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-values-filter-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-values-filter-map:{}", self.value)
    }
}

pub fn selected_btreemap_values_filter_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert("live", BtreemapValuesFilterMapPayload::new(raw));
    values.insert("empty", BtreemapValuesFilterMapPayload::new(""));
    values
        .values()
        .filter_map(|payload| (!payload.value.is_empty()).then(|| payload.render_label()))
        .next()
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_values_filter_map(raw: &str) -> String {
    BtreemapValuesFilterMapPayload::new(raw).unused_label()
}
