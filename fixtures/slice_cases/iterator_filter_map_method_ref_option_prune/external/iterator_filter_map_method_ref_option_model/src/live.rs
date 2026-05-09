pub struct IteratorFilterMapMethodRefOptionPayload {
    value: String,
}

impl IteratorFilterMapMethodRefOptionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_label(&self) -> Option<String> {
        (!self.value.is_empty()).then(|| self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("iterator-filter-map-method-ref-option:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-filter-map-method-ref-option:{}", self.value)
    }
}

pub fn selected_iterator_filter_map_method_ref_option(raw: &str) -> String {
    let items = vec![
        IteratorFilterMapMethodRefOptionPayload::new(""),
        IteratorFilterMapMethodRefOptionPayload::new(raw),
    ];
    items
        .iter()
        .filter_map(IteratorFilterMapMethodRefOptionPayload::maybe_label)
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_filter_map_method_ref_option(raw: &str) -> String {
    IteratorFilterMapMethodRefOptionPayload::new(raw).unused_label()
}
