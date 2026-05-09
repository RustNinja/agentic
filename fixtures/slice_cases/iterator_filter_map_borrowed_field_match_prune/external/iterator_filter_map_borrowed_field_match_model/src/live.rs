pub struct IteratorFilterMapBorrowedFieldMatchPayload {
    value: String,
}

impl IteratorFilterMapBorrowedFieldMatchPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-filter-map-borrowed-field-match:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!(
            "dead-iterator-filter-map-borrowed-field-match:{}",
            self.value
        )
    }
}

pub struct IteratorFilterMapBorrowedFieldMatchShadowPayload {
    value: String,
}

impl IteratorFilterMapBorrowedFieldMatchShadowPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }
}

pub struct IteratorFilterMapBorrowedFieldMatchEntry {
    path: Option<IteratorFilterMapBorrowedFieldMatchPayload>,
    shadow: Option<IteratorFilterMapBorrowedFieldMatchShadowPayload>,
}

impl IteratorFilterMapBorrowedFieldMatchEntry {
    pub fn live(raw: &str) -> Self {
        Self {
            path: Some(IteratorFilterMapBorrowedFieldMatchPayload::new(raw)),
            shadow: None,
        }
    }

    pub fn empty() -> Self {
        Self {
            path: None,
            shadow: None,
        }
    }
}

pub fn selected_iterator_filter_map_borrowed_field_match(raw: &str) -> String {
    let entries = vec![
        IteratorFilterMapBorrowedFieldMatchEntry::empty(),
        IteratorFilterMapBorrowedFieldMatchEntry::live(raw),
    ];
    entries
        .iter()
        .filter_map(|entry| match &entry.path {
            Some(payload) => Some(payload.render_label()),
            None => None,
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_filter_map_borrowed_field_match(raw: &str) -> String {
    IteratorFilterMapBorrowedFieldMatchPayload::new(raw).unused_label()
}
