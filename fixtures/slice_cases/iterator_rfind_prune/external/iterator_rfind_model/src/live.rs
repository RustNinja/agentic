pub struct IteratorRfindPayload {
    value: String,
}

impl IteratorRfindPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-rfind:{}", self.value)
    }

    pub fn bump_and_render(&mut self) -> String {
        self.value.push_str(":mut");
        format!("iterator-rfind:{}", self.value)
    }

    pub fn is_match(&self) -> bool {
        !self.value.is_empty()
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-rfind:{}", self.value)
    }
}

fn iterator_rfind_items(raw: &str) -> Vec<IteratorRfindPayload> {
    vec![IteratorRfindPayload::new("head"), IteratorRfindPayload::new(raw)]
}

pub fn selected_iterator_rfind(raw: &str) -> String {
    iterator_rfind_items(raw)
        .iter()
        .rfind(|payload| payload.is_match())
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "iterator-rfind:missing".to_string())
}

pub fn dead_live_iterator_rfind(raw: &str) -> String {
    IteratorRfindPayload::new(raw).unused_label()
}
