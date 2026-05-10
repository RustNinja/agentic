use std::collections::BTreeMap;
pub struct BtreemapClearInsertValuesMapPayload {
    value: String,
}

impl BtreemapClearInsertValuesMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-clear-insert-values-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-clear-insert-values-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-clear-insert-values-map:{}", self.value)
    }
}

pub fn selected_btreemap_clear_insert_values_map(raw: &str) -> String {
    let mut values = BTreeMap::new();
    values.insert(0usize, BtreemapClearInsertValuesMapPayload::new("old"));
    values.clear();
    values.insert(1usize, BtreemapClearInsertValuesMapPayload::new(raw));
    values
        .values()
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_default()
}

pub fn dead_live_btreemap_clear_insert_values_map(raw: &str) -> String {
    BtreemapClearInsertValuesMapPayload::new(raw).unused_label()
}
