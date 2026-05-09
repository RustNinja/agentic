pub struct IteratorFilterMapLetElseOptionPayload {
    value: String,
}

impl IteratorFilterMapLetElseOptionPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-filter-map-let-else-option:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-filter-map-let-else-option:{}", self.value)
    }
}

pub struct IteratorFilterMapLetElseOptionEntry {
    path: Option<IteratorFilterMapLetElseOptionPayload>,
}

impl IteratorFilterMapLetElseOptionEntry {
    pub fn live(raw: &str) -> Self {
        Self {
            path: Some(IteratorFilterMapLetElseOptionPayload::new(raw)),
        }
    }

    pub fn empty() -> Self {
        Self { path: None }
    }
}

pub fn selected_iterator_filter_map_let_else_option(raw: &str) -> String {
    let entries = vec![
        IteratorFilterMapLetElseOptionEntry::empty(),
        IteratorFilterMapLetElseOptionEntry::live(raw),
    ];
    entries
        .iter()
        .filter_map(|entry| {
            let Some(payload) = entry.path.as_ref() else {
                return None;
            };
            Some(payload.render_label())
        })
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_filter_map_let_else_option(raw: &str) -> String {
    IteratorFilterMapLetElseOptionPayload::new(raw).unused_label()
}
