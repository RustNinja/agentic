pub struct IteratorPeekablePeekMapPayload {
    value: String,
}

impl IteratorPeekablePeekMapPayload {
    pub fn new(raw: &str) -> Self {
        Self {
            value: raw.trim().to_string(),
        }
    }

    pub fn render_label(&self) -> String {
        format!("iterator-peekable-peek-map:{}", self.value)
    }

    pub fn unused_label(&self) -> String {
        format!("dead-iterator-peekable-peek-map:{}", self.value)
    }
}

pub fn selected_iterator_peekable_peek_map(raw: &str) -> String {
    let items = vec![IteratorPeekablePeekMapPayload::new(raw)];
    let mut iter = items.iter().peekable();
    iter.peek()
        .map(|payload| payload.render_label())
        .unwrap_or_else(|| "missing".to_string())
}

pub fn dead_live_iterator_peekable_peek_map(raw: &str) -> String {
    IteratorPeekablePeekMapPayload::new(raw).unused_label()
}
