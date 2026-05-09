use std::collections::BTreeMap;

pub struct BtreemapRangeMutFindMapPayload {
    value: String,
}

impl BtreemapRangeMutFindMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_label_mut(&mut self) -> Option<String> {
        self.value.push_str(":seen");
        (!self.value.is_empty()).then(|| self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-range-mut-find-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-range-mut-find-map:{}", self.value)
    }
}

pub fn selected_btreemap_range_mut_find_map(raw: &str) -> String {
    let mut map = BTreeMap::new();
    map.insert(1, BtreemapRangeMutFindMapPayload::new(raw));
    map.range_mut(0..=3)
        .find_map(|(_, payload)| payload.maybe_label_mut())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_range_mut_find_map(raw: &str) -> String {
    BtreemapRangeMutFindMapPayload::new(raw).unused_label()
}
