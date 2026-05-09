pub struct DeadIteratorFilterMapBorrowedFieldMatchItem {
    value: String,
}

impl DeadIteratorFilterMapBorrowedFieldMatchItem {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn dead_method(&self) -> String {
        format!(
            "dead-iterator-filter-map-borrowed-field-match:{}",
            self.value
        )
    }
}

pub fn dead_iterator_filter_map_borrowed_field_match(raw: &str) -> String {
    DeadIteratorFilterMapBorrowedFieldMatchItem::new(raw).dead_method()
}
