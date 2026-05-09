use std::collections::BTreeMap;

pub struct BtreemapRangeFindMapPayload {
    value: String,
}

impl BtreemapRangeFindMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_label(&self) -> Option<String> {
        (!self.value.is_empty()).then(|| self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-range-find-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-range-find-map:{}", self.value)
    }
}

pub fn selected_btreemap_range_find_map(raw: &str) -> String {
    let mut map = BTreeMap::new();
    map.insert(1, BtreemapRangeFindMapPayload::new(raw));
    map.range(0..=2)
        .find_map(|(_, payload)| payload.maybe_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_range_find_map(raw: &str) -> String {
    BtreemapRangeFindMapPayload::new(raw).unused_label()
}
