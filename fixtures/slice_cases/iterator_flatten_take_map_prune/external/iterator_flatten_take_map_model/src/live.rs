pub struct IteratorFlattenTakeMapPayload {
    value: String,
}

impl IteratorFlattenTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-flatten-take-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":bumped");
        self.render_label()
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-flatten-take-map:{}", self.value)
    }
}

pub fn selected_iterator_flatten_take_map(raw: &str) -> String {
    let items: Vec<Option<IteratorFlattenTakeMapPayload>> =
        vec![Some(IteratorFlattenTakeMapPayload::new(raw)), None];
    items
        .into_iter()
        .flatten()
        .take(1)
        .map(|payload| payload.render_label())
        .collect::<Vec<_>>()
        .join("|")
}

pub fn dead_live_iterator_flatten_take_map(raw: &str) -> String {
    IteratorFlattenTakeMapPayload::new(raw).dead_method()
}
