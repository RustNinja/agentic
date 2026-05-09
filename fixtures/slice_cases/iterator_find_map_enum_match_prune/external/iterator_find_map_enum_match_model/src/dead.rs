pub struct DeadIteratorFindMapEnumMatchItem {
    value: String,
}

impl DeadIteratorFindMapEnumMatchItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-find-map-enum-match:{}", self.value)
    }
}

pub fn dead_iterator_find_map_enum_match(raw: &str) -> String {
    DeadIteratorFindMapEnumMatchItem::new(raw).dead_method()
}
