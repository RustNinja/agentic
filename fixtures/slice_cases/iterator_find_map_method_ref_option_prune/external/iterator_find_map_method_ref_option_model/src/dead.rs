pub struct DeadIteratorFindMapMethodRefOptionItem {
    value: String,
}

impl DeadIteratorFindMapMethodRefOptionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-find-map-method-ref-option:{}", self.value)
    }
}

pub fn dead_iterator_find_map_method_ref_option(raw: &str) -> String {
    DeadIteratorFindMapMethodRefOptionItem::new(raw).dead_method()
}
