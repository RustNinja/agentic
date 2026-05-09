pub struct DeadBtreemapValuesFindMapMethodRefItem {
    value: String,
}

impl DeadBtreemapValuesFindMapMethodRefItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-btreemap-values-find-map-method-ref:{}", self.value)
    }
}

pub fn dead_btreemap_values_find_map_method_ref(raw: &str) -> String {
    DeadBtreemapValuesFindMapMethodRefItem::new(raw).dead_method()
}
