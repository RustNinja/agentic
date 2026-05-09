pub struct DeadIteratorFilterMapOptionIdentityItem {
    value: String,
}

impl DeadIteratorFilterMapOptionIdentityItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-filter-map-option-identity:{}", self.value)
    }
}

pub fn dead_iterator_filter_map_option_identity(raw: &str) -> String {
    DeadIteratorFilterMapOptionIdentityItem::new(raw).dead_method()
}
