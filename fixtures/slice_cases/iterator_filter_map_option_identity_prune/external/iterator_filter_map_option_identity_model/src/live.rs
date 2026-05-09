pub struct IteratorFilterMapOptionIdentityPayload {
    value: String,
}

impl IteratorFilterMapOptionIdentityPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-filter-map-option-identity:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-filter-map-option-identity:{}", self.value)
    }
}

pub fn selected_iterator_filter_map_option_identity(raw: &str) -> String {
    let items: Vec<Option<IteratorFilterMapOptionIdentityPayload>> =
        vec![Some(IteratorFilterMapOptionIdentityPayload::new(raw)), None];
    items
        .into_iter()
        .filter_map(|payload| payload)
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_filter_map_option_identity(raw: &str) -> String {
    IteratorFilterMapOptionIdentityPayload::new(raw).dead_method()
}
