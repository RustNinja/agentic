use std::collections::BTreeMap;
pub struct BtreemapValuesNextBackMapPayload {
    value: String,
}

impl BtreemapValuesNextBackMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-values-next-back-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("btreemap-values-next-back-map:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-values-next-back-map:{}", self.value)
    }
}

fn btreemap_values_next_back_map_items(raw: &str) -> BTreeMap<String, BtreemapValuesNextBackMapPayload> {
    let mut values = BTreeMap::new();
    values.insert("head".to_string(), BtreemapValuesNextBackMapPayload::new("head"));
    values.insert("tail".to_string(), BtreemapValuesNextBackMapPayload::new(raw));
    values
}

pub fn selected_btreemap_values_next_back_map(raw: &str) -> String {
    btreemap_values_next_back_map_items(raw)
        .values()
        .next_back()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "btreemap-values-next-back-map:missing".to_string())
}

pub fn dead_live_btreemap_values_next_back_map(raw: &str) -> String {
    BtreemapValuesNextBackMapPayload::new(raw).unused_label()
}
