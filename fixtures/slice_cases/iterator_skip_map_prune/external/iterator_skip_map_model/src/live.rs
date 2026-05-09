#[derive(Clone)]
pub struct IteratorSkipMapPayload {
    value: String,
}

impl IteratorSkipMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-skip-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("iterator-skip-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-skip-map:{}", self.value)
    }
}

pub fn selected_iterator_skip_map(raw: &str) -> String {
    let items = vec![
        IteratorSkipMapPayload::new("skip"),
        IteratorSkipMapPayload::new(raw),
    ];
    items
        .iter()
        .skip(1)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("iterator-skip-map:missing"))
}

pub fn dead_live_iterator_skip_map(raw: &str) -> String {
    let mut payload = IteratorSkipMapPayload::new(raw);
    payload.bump_and_render()
}
