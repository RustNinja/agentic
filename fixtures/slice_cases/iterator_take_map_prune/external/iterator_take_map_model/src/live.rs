#[derive(Clone)]
pub struct IteratorTakeMapPayload {
    value: String,
}

impl IteratorTakeMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-take-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("iterator-take-map:{}", self.value)
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-take-map:{}", self.value)
    }
}

pub fn selected_iterator_take_map(raw: &str) -> String {
    let items = vec![
        IteratorTakeMapPayload::new(raw),
        IteratorTakeMapPayload::new("dead"),
    ];
    items
        .iter()
        .take(1)
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("iterator-take-map:missing"))
}

pub fn dead_live_iterator_take_map(raw: &str) -> String {
    let mut payload = IteratorTakeMapPayload::new(raw);
    payload.bump_and_render()
}
