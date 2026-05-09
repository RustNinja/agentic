use std::collections::BTreeMap;

pub struct BtreemapValuesFindMapMethodRefPayload {
    value: String,
}

impl BtreemapValuesFindMapMethodRefPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_label(&self) -> Option<String> {
        (!self.value.is_empty()).then(|| self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("btreemap-values-find-map-method-ref:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-btreemap-values-find-map-method-ref:{}", self.value)
    }
}

pub fn selected_btreemap_values_find_map_method_ref(raw: &str) -> String {
    let mut entries: BTreeMap<String, BtreemapValuesFindMapMethodRefPayload> = BTreeMap::new();
    entries.insert(
        "empty".to_string(),
        BtreemapValuesFindMapMethodRefPayload::new(""),
    );
    entries.insert(
        "live".to_string(),
        BtreemapValuesFindMapMethodRefPayload::new(raw),
    );
    entries
        .values()
        .find_map(BtreemapValuesFindMapMethodRefPayload::maybe_label)
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_btreemap_values_find_map_method_ref(raw: &str) -> String {
    BtreemapValuesFindMapMethodRefPayload::new(raw).unused_label()
}
