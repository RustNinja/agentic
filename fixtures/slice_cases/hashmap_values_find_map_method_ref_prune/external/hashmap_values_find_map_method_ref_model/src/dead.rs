pub struct DeadHashmapValuesFindMapMethodRefItem {
    value: String,
}

impl DeadHashmapValuesFindMapMethodRefItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-hashmap-values-find-map-method-ref:{}", self.value)
    }
}

pub fn dead_hashmap_values_find_map_method_ref(raw: &str) -> String {
    DeadHashmapValuesFindMapMethodRefItem::new(raw).dead_method()
}
