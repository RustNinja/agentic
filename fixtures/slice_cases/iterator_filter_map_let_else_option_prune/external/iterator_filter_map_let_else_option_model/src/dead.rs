pub struct DeadIteratorFilterMapLetElseOptionItem {
    value: String,
}

impl DeadIteratorFilterMapLetElseOptionItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-filter-map-let-else-option:{}", self.value)
    }
}

pub fn dead_iterator_filter_map_let_else_option(raw: &str) -> String {
    DeadIteratorFilterMapLetElseOptionItem::new(raw).dead_method()
}
