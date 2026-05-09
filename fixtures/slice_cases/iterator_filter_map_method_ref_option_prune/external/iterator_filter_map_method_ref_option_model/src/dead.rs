pub struct DeadIteratorFilterMapMethodRefOptionItem {
    value: String,
}

impl DeadIteratorFilterMapMethodRefOptionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-filter-map-method-ref-option:{}", self.value)
    }
}

pub fn dead_iterator_filter_map_method_ref_option(raw: &str) -> String {
    DeadIteratorFilterMapMethodRefOptionItem::new(raw).dead_method()
}
