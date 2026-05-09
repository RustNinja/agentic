pub struct IteratorFindMapMethodRefOptionPayload {
    value: String,
}

impl IteratorFindMapMethodRefOptionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn maybe_label(&self) -> Option<String> {
        (!self.value.is_empty()).then(|| self.render_label())
    }

    pub fn render_label(&self) -> String {
        format!("iterator-find-map-method-ref-option:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-find-map-method-ref-option:{}", self.value)
    }
}

pub fn selected_iterator_find_map_method_ref_option(raw: &str) -> String {
    let items = vec![
        IteratorFindMapMethodRefOptionPayload::new(""),
        IteratorFindMapMethodRefOptionPayload::new(raw),
    ];
    items
        .iter()
        .find_map(IteratorFindMapMethodRefOptionPayload::maybe_label)
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_iterator_find_map_method_ref_option(raw: &str) -> String {
    IteratorFindMapMethodRefOptionPayload::new(raw).unused_label()
}
