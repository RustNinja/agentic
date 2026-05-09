pub struct IteratorFindMapEnumMatchPayload {
    value: String,
}

impl IteratorFindMapEnumMatchPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-find-map-enum-match:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-find-map-enum-match:{}", self.value)
    }
}

pub struct IteratorFindMapEnumMatchIgnoredPayload {
    value: String,
}

impl IteratorFindMapEnumMatchIgnoredPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.to_string(),
        }
    }

    pub fn ignored_label(&self) -> String {
        format!("ignored-iterator-find-map-enum-match:{}", self.value)
    }
}

pub enum IteratorFindMapEnumMatchEvent {
    Live(IteratorFindMapEnumMatchPayload),
    Ignored(IteratorFindMapEnumMatchIgnoredPayload),
    Empty,
}

pub fn selected_iterator_find_map_enum_match(raw: &str) -> String {
    let events = vec![
        IteratorFindMapEnumMatchEvent::Empty,
        IteratorFindMapEnumMatchEvent::Live(IteratorFindMapEnumMatchPayload::new(raw)),
        IteratorFindMapEnumMatchEvent::Ignored(IteratorFindMapEnumMatchIgnoredPayload::new(raw)),
    ];
    events
        .iter()
        .find_map(|event| match event {
            IteratorFindMapEnumMatchEvent::Live(payload) => Some(payload.render_label()),
            _ => None,
        })
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_iterator_find_map_enum_match(raw: &str) -> String {
    IteratorFindMapEnumMatchPayload::new(raw).unused_label()
}
