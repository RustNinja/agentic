#[derive(Clone)]
pub struct IteratorInspectMapPayload {
    value: String,
}

impl IteratorInspectMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-inspect-map:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":used");
        format!("iterator-inspect-map:{}", self.value)
    }

    pub fn touch(&self) {
        let _ = self.value.len();
    }

    pub fn dead_method(&self) -> String {
        format!("dead-iterator-inspect-map:{}", self.value)
    }
}

pub fn selected_iterator_inspect_map(raw: &str) -> String {
    let items = vec![IteratorInspectMapPayload::new(raw)];
    items
        .iter()
        .inspect(|payload| payload.touch())
        .map(|payload| payload.render_label())
        .next()
        .unwrap_or_else(|| format!("iterator-inspect-map:missing"))
}

pub fn dead_live_iterator_inspect_map(raw: &str) -> String {
    let mut payload = IteratorInspectMapPayload::new(raw);
    payload.bump_and_render()
}
